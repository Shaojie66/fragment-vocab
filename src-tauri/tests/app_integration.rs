use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{Duration, Utc};
use fragment_vocab_lib::{
    commands::{
        pet::{init_pet_on_startup, update_pet_on_study},
        review::{get_next_card_for_db, submit_review_for_db},
        wordbook::{delete_wordbook_for_db, list_wordbooks_for_db, set_wordbook_enabled_for_db},
        WordbookListItem,
    },
    db::{
        migration::Migrator, CardsRepository, Database, LogsRepository, PetsRepository,
        StateRepository, TagsRepository, WordbookImporter, WordsRepository,
    },
};

const EMBEDDED_WORDBOOK: &[u8] = include_bytes!("../../assets/wordbooks/ielts-core-3000.json");

struct TestDb {
    temp_dir: PathBuf,
    db: Database,
}

impl TestDb {
    fn new(test_name: &str) -> Self {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after UNIX_EPOCH")
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!(
            "fragment-vocab-{test_name}-{}-{unique_suffix}",
            std::process::id()
        ));
        fs::create_dir_all(&temp_dir).expect("failed to create temp dir");

        let db_path = temp_dir.join("fragment-vocab.db");
        let db = Database::new(db_path).expect("failed to create database");
        Migrator::run_migrations(&db).expect("failed to run migrations");

        Self { temp_dir, db }
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

fn import_json_wordbook(db: &Database, source: &str, file_name: &str, json: &str) {
    WordbookImporter::import_from_bytes(db, json.as_bytes(), source, Some(file_name))
        .expect("failed to import test wordbook");
}

fn find_wordbook<'a>(items: &'a [WordbookListItem], source: &str) -> &'a WordbookListItem {
    items
        .iter()
        .find(|item| item.source == source)
        .unwrap_or_else(|| panic!("missing wordbook source: {source}"))
}

fn assert_close(actual: f64, expected: f64) {
    let delta = (actual - expected).abs();
    assert!(
        delta < 1e-9,
        "expected {expected}, got {actual} (delta {delta})"
    );
}

#[test]
fn imports_embedded_wordbook_and_creates_srs_cards() {
    let ctx = TestDb::new("import-pipeline");

    let summary = WordbookImporter::import_from_bytes(
        &ctx.db,
        EMBEDDED_WORDBOOK,
        "ielts-core",
        Some("ielts-core-3000.json"),
    )
    .expect("embedded wordbook import should succeed");

    let words_repo = WordsRepository::new(ctx.db.get_connection());
    let cards_repo = CardsRepository::new(ctx.db.get_connection());

    assert_eq!(summary.format, "json");
    assert_eq!(summary.source, "ielts-core");
    assert!(summary.imported_count > 0);
    assert_eq!(summary.imported_count, summary.total_count);
    assert_eq!(summary.skipped_count, 0);
    assert_eq!(words_repo.count().unwrap(), summary.imported_count as i64);
    assert_eq!(
        cards_repo.count_by_status("new").unwrap(),
        summary.imported_count as i64
    );

    let abandon = words_repo
        .get_by_word("abandon")
        .unwrap()
        .expect("abandon should exist after import");
    let abandon_card = cards_repo
        .get_by_word_id(abandon.id)
        .unwrap()
        .expect("abandon card should exist after import");

    assert_eq!(abandon.meaning_zh, "放弃；抛弃");
    assert_eq!(abandon.source, "ielts-core");
    assert_eq!(abandon_card.status, "new");
    assert_eq!(abandon_card.stage, -1);
    assert!(abandon_card.due_at.is_none());
}

#[test]
fn review_flow_selects_new_card_and_updates_srs_state() {
    let ctx = TestDb::new("review-flow");

    import_json_wordbook(
        &ctx.db,
        "custom-review",
        "review.json",
        r#"[
            {"word":"alpha","meaning_zh":"阿尔法","difficulty":1},
            {"word":"bravo","meaning_zh":"布拉沃","difficulty":1},
            {"word":"charlie","meaning_zh":"查理","difficulty":1},
            {"word":"delta","meaning_zh":"德尔塔","difficulty":1}
        ]"#,
    );

    let card = get_next_card_for_db(&ctx.db)
        .unwrap()
        .expect("expected a new card after import");
    assert_eq!(card.word, "alpha");
    assert_eq!(card.options.len(), 4);
    assert_eq!(card.correct_option_id, format!("word-{}", card.word_id));

    submit_review_for_db(&ctx.db, card.card_id, "know").expect("review submission should work");

    let cards_repo = CardsRepository::new(ctx.db.get_connection());
    let logs_repo = LogsRepository::new(ctx.db.get_connection());
    let pets_repo = PetsRepository::new(ctx.db.get_connection());
    let updated_card = cards_repo
        .get_by_id(card.card_id)
        .unwrap()
        .expect("reviewed card should still exist");
    let logs = logs_repo.get_by_card_id(card.card_id, 10).unwrap();
    let pet = pets_repo.get_or_create().unwrap();

    assert_eq!(updated_card.status, "learning");
    assert_eq!(updated_card.stage, 0);
    assert_eq!(updated_card.correct_streak, 1);
    assert_eq!(updated_card.lifetime_correct, 1);
    assert_eq!(updated_card.lifetime_wrong, 0);
    assert_eq!(updated_card.last_result.as_deref(), Some("know"));
    assert!(updated_card.last_seen_at.is_some());
    assert!(updated_card.due_at.is_some());

    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].result, "know");
    assert_eq!(logs[0].trigger_type, "manual");

    assert_eq!(pet.experience, 1);
    assert_close(pet.health, 1.0);
    assert!(pet.last_study_at.is_some());
}

#[test]
fn wordbook_management_updates_enabled_state_and_deletes_sources() {
    let ctx = TestDb::new("wordbook-management");

    import_json_wordbook(
        &ctx.db,
        "custom-alpha",
        "alpha.json",
        r#"[
            {"word":"alpha-one","meaning_zh":"甲一","difficulty":1},
            {"word":"alpha-two","meaning_zh":"甲二","difficulty":1}
        ]"#,
    );
    import_json_wordbook(
        &ctx.db,
        "custom-beta",
        "beta.json",
        r#"[
            {"word":"beta-one","meaning_zh":"乙一","difficulty":1},
            {"word":"beta-two","meaning_zh":"乙二","difficulty":1}
        ]"#,
    );

    let listed = list_wordbooks_for_db(&ctx.db).expect("listing wordbooks should work");
    assert_eq!(listed.len(), 2);
    assert!(find_wordbook(&listed, "custom-alpha").enabled);
    assert!(find_wordbook(&listed, "custom-beta").enabled);

    let disabled = set_wordbook_enabled_for_db(&ctx.db, "custom-alpha", false)
        .expect("disabling a wordbook should work");
    assert!(!find_wordbook(&disabled, "custom-alpha").enabled);

    let next_with_alpha_disabled = get_next_card_for_db(&ctx.db)
        .unwrap()
        .expect("beta cards should still be selectable");
    let words_repo = WordsRepository::new(ctx.db.get_connection());
    let selected_word = words_repo
        .get_by_id(next_with_alpha_disabled.word_id)
        .unwrap()
        .expect("selected word should exist");
    assert_eq!(selected_word.source, "custom-beta");

    let reenabled = set_wordbook_enabled_for_db(&ctx.db, "custom-alpha", true)
        .expect("re-enabling a wordbook should work");
    assert!(find_wordbook(&reenabled, "custom-alpha").enabled);

    let next_with_alpha_enabled = get_next_card_for_db(&ctx.db)
        .unwrap()
        .expect("alpha cards should be selectable again");
    let selected_word = words_repo
        .get_by_id(next_with_alpha_enabled.word_id)
        .unwrap()
        .expect("selected word should exist");
    assert_eq!(selected_word.source, "custom-alpha");

    let remaining = delete_wordbook_for_db(&ctx.db, "custom-beta")
        .expect("deleting a custom wordbook should work");
    let state_repo = StateRepository::new(ctx.db.get_connection());
    let cards_repo = CardsRepository::new(ctx.db.get_connection());

    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].source, "custom-alpha");
    assert!(remaining[0].enabled);
    assert_eq!(words_repo.count().unwrap(), 2);
    assert_eq!(cards_repo.count_by_status("new").unwrap(), 2);
    assert_eq!(
        state_repo.get("disabled_wordbook_sources").unwrap(),
        Some("[]".to_string())
    );
}

#[test]
fn pet_startup_and_study_actions_update_health_and_experience() {
    let ctx = TestDb::new("pet-system");

    let pets_repo = PetsRepository::new(ctx.db.get_connection());
    let mut pet = pets_repo.get_or_create().unwrap();
    pet.health = 1.0;
    pet.experience = 0;
    pet.current_streak = 10;
    pet.vitality_multiplier = 1.5;
    pet.last_study_at = Some((Utc::now() - Duration::days(2)).to_rfc3339());
    pet.last_review_at = Some((Utc::now() - Duration::days(8)).to_rfc3339());
    pets_repo.update(&pet).unwrap();

    init_pet_on_startup(&ctx.db).expect("startup pet init should succeed");

    let pet_after_startup = pets_repo.get_or_create().unwrap();
    assert_close(pet_after_startup.health, 0.5);
    assert_eq!(pet_after_startup.current_streak, 0);
    assert_close(pet_after_startup.vitality_multiplier, 1.0);

    let pet_after_study = update_pet_on_study(&ctx.db).expect("study action should update pet");
    assert_close(pet_after_study.health, 0.55);
    assert_eq!(pet_after_study.experience, 1);
    assert_eq!(pet_after_study.current_streak, 1);
    assert_close(pet_after_study.vitality_multiplier, 1.0);
    assert!(pet_after_study.last_study_at.is_some());
}

#[test]
fn tag_management_create_assign_filter_delete() {
    let ctx = TestDb::new("tag-management");

    import_json_wordbook(
        &ctx.db,
        "custom-tagged",
        "tagged.json",
        r#"[
            {"word":"apple","meaning_zh":"苹果","difficulty":1},
            {"word":"banana","meaning_zh":"香蕉","difficulty":1},
            {"word":"cherry","meaning_zh":"樱桃","difficulty":1}
        ]"#,
    );

    let words_repo = WordsRepository::new(ctx.db.get_connection());
    let tags_repo = TagsRepository::new(ctx.db.get_connection());

    // Create tags
    let fruit_tag = tags_repo.create("水果").expect("should create tag");
    let fav_tag = tags_repo.create("最爱").expect("should create second tag");
    assert_eq!(fruit_tag.name, "水果");
    assert_eq!(fav_tag.name, "最爱");

    // List tags (should show 0 words each)
    let all_tags = tags_repo.list_with_counts().unwrap();
    assert_eq!(all_tags.len(), 2);
    assert!(all_tags.iter().all(|t| t.word_count == 0));

    // Assign tags to words
    let apple = words_repo.get_by_word("apple").unwrap().unwrap();
    let banana = words_repo.get_by_word("banana").unwrap().unwrap();

    tags_repo.add_word_tag(apple.id, fruit_tag.id).unwrap();
    tags_repo.add_word_tag(banana.id, fruit_tag.id).unwrap();
    tags_repo.add_word_tag(apple.id, fav_tag.id).unwrap();

    // Verify word tags
    let apple_tags = tags_repo.get_word_tags(apple.id).unwrap();
    assert_eq!(apple_tags.len(), 2);

    let banana_tags = tags_repo.get_word_tags(banana.id).unwrap();
    assert_eq!(banana_tags.len(), 1);
    assert_eq!(banana_tags[0].name, "水果");

    // Verify counts
    let all_tags = tags_repo.list_with_counts().unwrap();
    let fruit = all_tags.iter().find(|t| t.name == "水果").unwrap();
    let fav = all_tags.iter().find(|t| t.name == "最爱").unwrap();
    assert_eq!(fruit.word_count, 2);
    assert_eq!(fav.word_count, 1);

    // List words by tag
    let fruit_words = tags_repo.list_words_by_tag(fruit_tag.id).unwrap();
    assert_eq!(fruit_words.len(), 2);

    // Remove a word-tag association
    tags_repo.remove_word_tag(apple.id, fav_tag.id).unwrap();
    let apple_tags = tags_repo.get_word_tags(apple.id).unwrap();
    assert_eq!(apple_tags.len(), 1);

    // Delete a tag (should cascade)
    tags_repo.delete(fruit_tag.id).unwrap();
    let remaining_tags = tags_repo.list_with_counts().unwrap();
    assert_eq!(remaining_tags.len(), 1);
    assert_eq!(remaining_tags[0].name, "最爱");

    // Word should have no tags after fruit tag deleted
    let apple_tags = tags_repo.get_word_tags(apple.id).unwrap();
    assert!(apple_tags.is_empty());
}

#[test]
fn delete_wordbook_rejects_builtin_source() {
    let ctx = TestDb::new("delete-builtin");

    let result = delete_wordbook_for_db(&ctx.db, "ielts-core");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("内置词库不能删除"));
}
