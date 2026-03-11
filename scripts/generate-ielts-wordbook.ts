/**
 * 生成雅思核心词库 JSON 文件
 * 
 * 词库来源：基于雅思考试高频词汇整理
 * 包含音标、词性、中文释义
 */

interface Word {
  word: string;
  phonetic: string;
  part_of_speech: string;
  meaning_zh: string;
  difficulty: number;
  source: string;
}

// 雅思核心词汇 - 按难度和主题分类
// 这里提供一个示例结构，实际词库需要从权威来源补充完整
const ieltsVocabulary: Word[] = [
  // Academic & Education (学术教育类)
  { word: "abandon", phonetic: "/əˈbændən/", part_of_speech: "v.", meaning_zh: "放弃;抛弃", difficulty: 1, source: "ielts-core" },
  { word: "ability", phonetic: "/əˈbɪləti/", part_of_speech: "n.", meaning_zh: "能力;才能", difficulty: 1, source: "ielts-core" },
  { word: "abstract", phonetic: "/ˈæbstrækt/", part_of_speech: "adj.", meaning_zh: "抽象的;理论的", difficulty: 2, source: "ielts-core" },
  { word: "academic", phonetic: "/ˌækəˈdemɪk/", part_of_speech: "adj.", meaning_zh: "学术的;理论的", difficulty: 1, source: "ielts-core" },
  { word: "accelerate", phonetic: "/əkˈseləreɪt/", part_of_speech: "v.", meaning_zh: "加速;促进", difficulty: 2, source: "ielts-core" },
  { word: "access", phonetic: "/ˈækses/", part_of_speech: "n./v.", meaning_zh: "接近;进入;使用", difficulty: 1, source: "ielts-core" },
  { word: "accommodate", phonetic: "/əˈkɒmədeɪt/", part_of_speech: "v.", meaning_zh: "容纳;适应;提供住宿", difficulty: 2, source: "ielts-core" },
  { word: "accomplish", phonetic: "/əˈkʌmplɪʃ/", part_of_speech: "v.", meaning_zh: "完成;实现", difficulty: 2, source: "ielts-core" },
  { word: "accumulate", phonetic: "/əˈkjuːmjəleɪt/", part_of_speech: "v.", meaning_zh: "积累;堆积", difficulty: 2, source: "ielts-core" },
  { word: "accurate", phonetic: "/ˈækjərət/", part_of_speech: "adj.", meaning_zh: "准确的;精确的", difficulty: 1, source: "ielts-core" },
  
  // Environment & Nature (环境自然类)
  { word: "adapt", phonetic: "/əˈdæpt/", part_of_speech: "v.", meaning_zh: "适应;改编", difficulty: 1, source: "ielts-core" },
  { word: "adequate", phonetic: "/ˈædɪkwət/", part_of_speech: "adj.", meaning_zh: "足够的;适当的", difficulty: 2, source: "ielts-core" },
  { word: "adjacent", phonetic: "/əˈdʒeɪsnt/", part_of_speech: "adj.", meaning_zh: "邻近的;毗连的", difficulty: 2, source: "ielts-core" },
  { word: "adjust", phonetic: "/əˈdʒʌst/", part_of_speech: "v.", meaning_zh: "调整;适应", difficulty: 1, source: "ielts-core" },
  { word: "administration", phonetic: "/ədˌmɪnɪˈstreɪʃn/", part_of_speech: "n.", meaning_zh: "管理;行政", difficulty: 2, source: "ielts-core" },
  { word: "advocate", phonetic: "/ˈædvəkeɪt/", part_of_speech: "v./n.", meaning_zh: "提倡;拥护者", difficulty: 2, source: "ielts-core" },
  { word: "aesthetic", phonetic: "/iːsˈθetɪk/", part_of_speech: "adj.", meaning_zh: "美学的;审美的", difficulty: 3, source: "ielts-core" },
  { word: "affect", phonetic: "/əˈfekt/", part_of_speech: "v.", meaning_zh: "影响;感染", difficulty: 1, source: "ielts-core" },
  { word: "agriculture", phonetic: "/ˈæɡrɪkʌltʃə(r)/", part_of_speech: "n.", meaning_zh: "农业;农学", difficulty: 1, source: "ielts-core" },
  { word: "allocate", phonetic: "/ˈæləkeɪt/", part_of_speech: "v.", meaning_zh: "分配;拨出", difficulty: 2, source: "ielts-core" },
  
  // Technology & Science (科技科学类)
  { word: "alternative", phonetic: "/ɔːlˈtɜːnətɪv/", part_of_speech: "adj./n.", meaning_zh: "可供选择的;替代物", difficulty: 1, source: "ielts-core" },
  { word: "ambiguous", phonetic: "/æmˈbɪɡjuəs/", part_of_speech: "adj.", meaning_zh: "模糊的;含糊的", difficulty: 3, source: "ielts-core" },
  { word: "analyze", phonetic: "/ˈænəlaɪz/", part_of_speech: "v.", meaning_zh: "分析;解析", difficulty: 1, source: "ielts-core" },
  { word: "ancient", phonetic: "/ˈeɪnʃənt/", part_of_speech: "adj.", meaning_zh: "古代的;古老的", difficulty: 1, source: "ielts-core" },
  { word: "annual", phonetic: "/ˈænjuəl/", part_of_speech: "adj.", meaning_zh: "每年的;年度的", difficulty: 1, source: "ielts-core" },
  { word: "anticipate", phonetic: "/ænˈtɪsɪpeɪt/", part_of_speech: "v.", meaning_zh: "预期;期望", difficulty: 2, source: "ielts-core" },
  { word: "apparent", phonetic: "/əˈpærənt/", part_of_speech: "adj.", meaning_zh: "明显的;表面上的", difficulty: 2, source: "ielts-core" },
  { word: "appeal", phonetic: "/əˈpiːl/", part_of_speech: "v./n.", meaning_zh: "呼吁;吸引力", difficulty: 1, source: "ielts-core" },
  { word: "approach", phonetic: "/əˈprəʊtʃ/", part_of_speech: "v./n.", meaning_zh: "接近;方法", difficulty: 1, source: "ielts-core" },
  { word: "appropriate", phonetic: "/əˈprəʊpriət/", part_of_speech: "adj.", meaning_zh: "适当的;恰当的", difficulty: 1, source: "ielts-core" },
  
  // 注意：这只是示例数据结构
  // 完整的3000词需要从权威雅思词库补充
];

// 生成完整词库的函数
function generateWordbook(): void {
  const wordbook = {
    meta: {
      name: "IELTS Core 3000",
      description: "雅思核心词汇3000词",
      version: "1.0.0",
      total: ieltsVocabulary.length,
      source: "ielts-core",
      created_at: new Date().toISOString(),
      difficulty_levels: {
        1: "基础词汇",
        2: "核心词汇",
        3: "高级词汇"
      }
    },
    words: ieltsVocabulary
  };

  console.log(JSON.stringify(wordbook, null, 2));
}

// 执行生成
generateWordbook();

/**
 * 使用说明：
 * 
 * 1. 补充完整词库数据
 *    - 从权威雅思词汇书或在线资源获取
 *    - 确保音标准确（使用IPA国际音标）
 *    - 中文释义简洁明确
 * 
 * 2. 运行脚本生成JSON
 *    npx tsx scripts/generate-ielts-wordbook.ts > assets/wordbooks/ielts-core-3000.json
 * 
 * 3. 词库质量要求
 *    - 音标格式统一
 *    - 词性标注规范（n./v./adj./adv.等）
 *    - 中文释义控制在15字以内
 *    - 难度分级合理
 * 
 * 4. 扩展建议
 *    - 可添加例句字段
 *    - 可添加同义词/反义词
 *    - 可添加词根词缀信息
 *    - 可添加使用频率标记
 */
