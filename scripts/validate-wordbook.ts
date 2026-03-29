#!/usr/bin/env npx tsx
/**
 * Wordbook validation script
 * Usage: npx tsx scripts/validate-wordbook.ts <file.json>
 *
 * Checks:
 * - JSON format correctness
 * - Required fields (word, meaning_zh) present on every entry
 * - Duplicate word detection
 * - Reports summary
 */

import { readFileSync } from "fs";
import { resolve } from "path";

const WORD_ALIASES = new Set([
  "word",
  "english",
  "term",
  "vocab",
  "vocabulary",
]);
const MEANING_ALIASES = new Set([
  "meaning_zh",
  "meaning",
  "translation",
  "definition",
  "chinese",
]);

interface ValidationResult {
  valid: boolean;
  totalEntries: number;
  errors: string[];
  warnings: string[];
  duplicates: string[];
}

function validate(filePath: string): ValidationResult {
  const result: ValidationResult = {
    valid: true,
    totalEntries: 0,
    errors: [],
    warnings: [],
    duplicates: [],
  };

  // Read file
  let raw: string;
  try {
    raw = readFileSync(filePath, "utf-8");
  } catch (e: any) {
    result.valid = false;
    result.errors.push(`Cannot read file: ${e.message}`);
    return result;
  }

  // Parse JSON
  let data: any;
  try {
    data = JSON.parse(raw);
  } catch (e: any) {
    result.valid = false;
    result.errors.push(`Invalid JSON: ${e.message}`);
    return result;
  }

  // Normalize: support array or { words: [...] } wrapper
  let entries: any[];
  if (Array.isArray(data)) {
    entries = data;
  } else if (data && typeof data === "object") {
    const key = ["words", "entries", "items", "vocabulary", "data", "list"].find(
      (k) => Array.isArray(data[k])
    );
    if (key) {
      entries = data[key];
    } else {
      result.valid = false;
      result.errors.push(
        "JSON root must be an array or an object with a words/entries/items array"
      );
      return result;
    }
  } else {
    result.valid = false;
    result.errors.push("JSON root must be an array or object");
    return result;
  }

  result.totalEntries = entries.length;

  if (entries.length === 0) {
    result.warnings.push("Wordbook is empty (0 entries)");
    return result;
  }

  // Detect field names from first entry
  const firstKeys = Object.keys(entries[0]).map((k) => k.toLowerCase());
  const hasWordField = firstKeys.some((k) => WORD_ALIASES.has(k));
  const hasMeaningField = firstKeys.some((k) => MEANING_ALIASES.has(k));

  if (!hasWordField) {
    result.valid = false;
    result.errors.push(
      `No word field found. Expected one of: ${[...WORD_ALIASES].join(", ")}`
    );
  }
  if (!hasMeaningField) {
    result.valid = false;
    result.errors.push(
      `No meaning field found. Expected one of: ${[...MEANING_ALIASES].join(", ")}`
    );
  }

  if (!hasWordField || !hasMeaningField) {
    return result;
  }

  // Find actual field names
  const wordKey = Object.keys(entries[0]).find((k) =>
    WORD_ALIASES.has(k.toLowerCase())
  )!;
  const meaningKey = Object.keys(entries[0]).find((k) =>
    MEANING_ALIASES.has(k.toLowerCase())
  )!;

  // Validate each entry
  const seen = new Map<string, number>();
  let missingWord = 0;
  let missingMeaning = 0;
  let emptyWord = 0;
  let emptyMeaning = 0;

  for (let i = 0; i < entries.length; i++) {
    const entry = entries[i];
    if (typeof entry !== "object" || entry === null) {
      result.errors.push(`Entry ${i + 1}: not an object`);
      result.valid = false;
      continue;
    }

    const word = entry[wordKey];
    const meaning = entry[meaningKey];

    if (word === undefined || word === null) {
      missingWord++;
    } else if (typeof word === "string" && word.trim() === "") {
      emptyWord++;
    }

    if (meaning === undefined || meaning === null) {
      missingMeaning++;
    } else if (typeof meaning === "string" && meaning.trim() === "") {
      emptyMeaning++;
    }

    if (typeof word === "string" && word.trim()) {
      const normalized = word.trim().toLowerCase();
      if (seen.has(normalized)) {
        result.duplicates.push(
          `"${word.trim()}" (entries ${seen.get(normalized)! + 1} and ${i + 1})`
        );
      } else {
        seen.set(normalized, i);
      }
    }
  }

  if (missingWord > 0) {
    result.valid = false;
    result.errors.push(`${missingWord} entries missing "${wordKey}" field`);
  }
  if (emptyWord > 0) {
    result.warnings.push(`${emptyWord} entries have empty "${wordKey}"`);
  }
  if (missingMeaning > 0) {
    result.valid = false;
    result.errors.push(
      `${missingMeaning} entries missing "${meaningKey}" field`
    );
  }
  if (emptyMeaning > 0) {
    result.warnings.push(
      `${emptyMeaning} entries have empty "${meaningKey}"`
    );
  }
  if (result.duplicates.length > 0) {
    result.warnings.push(
      `${result.duplicates.length} duplicate word(s) found`
    );
  }

  return result;
}

// --- Main ---

const args = process.argv.slice(2);
if (args.length === 0) {
  console.log("Usage: npx tsx scripts/validate-wordbook.ts <wordbook.json>");
  process.exit(1);
}

const filePath = resolve(args[0]);
console.log(`Validating: ${filePath}\n`);

const result = validate(filePath);

console.log(`Total entries: ${result.totalEntries}`);
console.log(`Status: ${result.valid ? "VALID" : "INVALID"}\n`);

if (result.errors.length > 0) {
  console.log("Errors:");
  result.errors.forEach((e) => console.log(`  - ${e}`));
  console.log();
}

if (result.warnings.length > 0) {
  console.log("Warnings:");
  result.warnings.forEach((w) => console.log(`  - ${w}`));
  console.log();
}

if (result.duplicates.length > 0) {
  console.log("Duplicates:");
  result.duplicates.slice(0, 20).forEach((d) => console.log(`  - ${d}`));
  if (result.duplicates.length > 20) {
    console.log(`  ... and ${result.duplicates.length - 20} more`);
  }
  console.log();
}

process.exit(result.valid ? 0 : 1);
