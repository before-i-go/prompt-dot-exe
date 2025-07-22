# 🧙‍♂️ Interview Irodov

*"Ah, my dear student of the digital arts! Within these virtual halls lies a most curious collection of knowledge - both the ancient runes of interview preparation and a rather nifty little tool for taming the wild TypeScript. Do tread carefully, for wisdom and magic await those who seek them."*

*Albus Dumbledore's Repository of Magical Coding Artifacts and Interview Enchantments*

## 🧰 The Marauder's Compressor

*"A most ingenious contraption, wouldn't you agree? With a wave of your terminal and the right incantation, it transforms verbose TypeScript into something rather more... compact. Much like how Fawkes can fit into a small cage, yet remain a magnificent phoenix at heart."*

### 🪄 Compilation Charm

```bash
cd ts-compressor
cargo build --release  # The modern wizard's equivalent of 'Wingardium Leviosa'
```

*"A word to the wise: This spell requires Rust 1.70 or later, and the ever-useful Git - though I daresay you've already made its acquaintance."*

### 📜 The Spellbook of Commands

**The Condensing Charm (TypeScript Edition):**
```bash
./target/release/ts-compressor compress src/ dist/
```

*"A clever bit of magic that transforms your verbose TypeScript into something more... portable. It handles both `.ts` and `.tsx` scrolls, stripping away the type annotations and compressing the rest - much like how one might summarize a particularly long-winded prophecy."*

**The Archive Charm (Project Preservation Spell):**
```bash
./target/release/ts-compressor archive my-project
```

Sample output creates: `my-project-20250118142033.txt`

```
Git repository detected. Will respect .gitignore rules.

Directory structure:
├── src
│   ├── main.ts
│   └── utils.ts
├── package.json
└── README.md

Processing files...
🤖 LLM optimization enabled - excluding build artifacts and dependencies

Absolute path: /home/user/my-project/src/main.ts
<text starts>
interface User {
    name: string;
    age: number;
}
<text ends>

Absolute path: /home/user/my-project/package.json
<text starts>
{
  "name": "my-project",
  "version": "1.0.0"
}
<text ends>

📊 File Filtering Statistics:
   Total files found: 247
   Files included: 23 🟢
   Files excluded: 224 🔴
     └─ By LLM optimization: 224 🤖
   Inclusion rate: 9.3% 📈
   Total size included: 1.2 MB 💾

Archive created: "my-project-20250118142033.txt"
```

### 🏰 Default Enchantments

- **The Wisdom of the Ancients (LLM Optimization)**: Like the Room of Requirement, it knows what to hide - build artifacts, dependencies, and cache files vanish from sight
- **The Keeper's Memory (Git Integration)**: Respects the sacred `.gitignore` scrolls, just as we respect the boundaries of the Forbidden Forest
- **The Revealing Charm (Binary Detection)**: Spots and excludes binary files with the precision of a Niffler spotting gold
- **The Time-Turner Feature**: Creates uniquely timestamped archives, because even wizards need to keep track of their past exploits

### Command Options

```bash
# Disable LLM optimization (includes all files)
ts-compressor archive my-project --no-llm-optimize

# Custom ignore patterns
ts-compressor archive my-project --ignore-pattern "*.tmp" --ignore-pattern "test_*"

# Filter by extensions
ts-compressor archive my-project --include-extensions rs,js,ts,md

# Hide filtering statistics
ts-compressor archive my-project --no-filter-stats

# Custom output directory
ts-compressor archive my-project --output-dir ./archives
```

### 🚫 The Forbidden Files (Vanished by LLM Optimization)

*"Even the most powerful wizards know that not all files are created equal. These are banished to the depths of the Room of Requirement, never to trouble your archives:"*

- **The Build Cauldron's Residue**: `target/`, `build/`, `dist/`, `*.exe`, `*.dll`
- **Dependency Demons**: `node_modules/`, `vendor/`, `venv/` (No need to carry around other wizards' spellbooks)
- **Temporal Echoes**: `.cache/`, `*.tmp`, `*.bak` (The Pensieve has its limits)
- **Muggle Artifacts**: `.DS_Store`, `Thumbs.db` (We must respect the Statute of Secrecy)
- **Moving Portraits**: `*.png`, `*.jpg`, `*.mp4` (Alas, they don't move in text form)
- **Binding Contracts**: `package-lock.json`, `Cargo.lock` (Some things are better left unbound)

### 🌟 Practical Applications for the Discerning Wizard

*"While not quite as versatile as a wand that can turn teacups into turtles, this tool serves several rather useful purposes:"*

- **The Pensieve Effect**: Create complete project snapshots in text format for later perusal
- **Occlumency for Code**: Prepare your spells—er, code—for LLM analysis by removing the mental clutter
- **The Shrinking Solution**: Minify TypeScript for deployment (without the unfortunate side effects of actual shrinking)
- **The Mirror of Erised**: Review and document your code to see it as it truly is, not as you wish it to be
- **The Vanishing Cabinet**: Safely archive project states, ready to be recalled at a moment's notice

## 🧪 Experimental Charms (Testing)

*"Before unleashing any magical artifact upon the world, one must first test it thoroughly. The Department of Mysteries suggests the following incantations:"*

```bash
cd ts-compressor
cargo test  # The Standard Book of Spells, Testing Edition
cargo run -- --help  # Consult the ancient scrolls
cargo run -- archive ../test-input  # A small sacrifice to the testing gods
```

## 🏰 The Castle Layout

*"Every great wizard's tower has its secrets, and this repository is no exception. Here's what lies within these digital walls:"*

```
├── ts-compressor/           # The Marauder's Map of Code Compression
│   ├── src/main.rs         # The Sorcerer's Stone (core logic)
│   ├── Cargo.toml          # The Potion Master's recipe book
│   └── tests/              # The Triwizard Tournament (challenges await!)
├── test-input/             # The Room of Requirement (for testing)
│   └── example.ts          # A prophecy yet to be fulfilled
├── zzArchive/              # The Restricted Section
│   ├── RailsCrashCours202507.ipynb    # The Tales of Beedle the Bard (Rails edition)
│   └── RustCrashCourse202507.ipynb    # Advanced Rune Studies
├── Unclassified20250706.txt # The Half-Blood Prince's Notes
├── i00-pattern-list.txt    # The Standard Book of Spells (Interview Edition)
└── split_large_file.sh     # The Sword of Gryffindor (for cutting large files)
```

## 🧙‍♂️ Magical Ingredients

*"No spell is complete without the right components. These are the enchanted artifacts that make our magic possible:"*

- **swc_core**: The Elder Wand of TypeScript compilation
- **clap**: The Marauder's Map for command-line arguments
- **git2**: A loyal house-elf for Git repository integration
- **walkdir**: The Invisibility Cloak for directory traversal
- **tracing**: The Pensieve for structured logging
- **mime_guess**: The Sorting Hat of file type detection

## 📚 The Restricted Section

*"For your O.W.L.s and N.E.W.T.s in the magical arts of coding, I present these most valuable resources:"*

- `zzArchive/`: The collected works of modern arithmancy (Rails and Rust)
- `Unclassified20250706.txt`: Mysterious prophecies (interview questions) yet to be deciphered
- `i00-pattern-list.txt`: Ancient runes of coding patterns (handy for defeating your technical interviews)
- Various `.md` scrolls containing the collective wisdom of wizards past

*"Remember, it does not do to dwell on dreams and forget to live... but a little preparation never hurt anyone. Now, off you go - I believe you have some code to write?"*