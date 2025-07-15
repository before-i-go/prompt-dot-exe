# Interview Prep Repository Structure

## Repository Organization

This repository follows a flat structure optimized for interview preparation materials and quick reference during study sessions.

### Root Level Files

```
├── README.md                     # Repository overview and study guide
├── RailsCrashCours202507.ipynb   # Rails interview questions and concepts
├── RailsViaRust20250707.txt      # Backend framework comparisons and deep-dive questions
├── RustCrashCourse202507.ipynb   # Rust systems programming interview prep
└── Unclassified20250706.txt      # Mixed technical concepts and behavioral questions
```

### Hidden Directories

```
├── .git/                         # Git version control metadata
└── .kiro/                        # Kiro AI assistant configuration
    └── steering/                 # AI guidance for interview prep assistance
        ├── product.md            # Interview prep focus and target roles
        ├── tech.md               # Technology categories and question types
        └── structure.md          # This file - organization principles
```

## File Naming Conventions

### Date-based Organization
- Format: `[Technology/Topic][YYYYMMDD].[extension]`
- Examples: `RailsCrashCours202507.ipynb`, `RustCrashCourse202507.ipynb`
- Purpose: Track learning progression and interview prep timeline

### Content Categories

#### Interactive Study Materials
- **Jupyter Notebooks** (`.ipynb`): Coding practice, algorithm implementations, interactive Q&A
- **Text Files** (`.txt`): Comprehensive question banks, system design scenarios, concept explanations
- **Markdown Files** (`.md`): Structured study guides, checklists, and reference materials

#### Question Types by File
- **Technology-Specific**: Deep technical questions for specific frameworks/languages
- **System Design**: Architecture and scalability interview scenarios
- **Mixed/General**: Cross-cutting concepts, behavioral questions, general CS fundamentals

## Interview Prep Organization Principles

### Flat Structure Benefits
- **Quick Access**: All materials visible at root level for rapid review
- **Search Friendly**: Easy to grep across all files for specific topics
- **Session-Based**: Can quickly open relevant files for focused study sessions

### Question-Driven Learning
- **Format Questions as Headers**: Use "Q:" or "What is..." to make questions searchable
- **Include Difficulty Levels**: Mark questions as Junior/Mid/Senior level
- **Cross-Reference**: Link related concepts across different technology files

## Study Session Workflow

### Daily Prep Routine
1. **Review Questions**: `grep -r "Q:" *.txt *.md` to find all questions
2. **Topic Focus**: Open specific technology file for deep dive
3. **Practice Coding**: Use Jupyter notebooks for hands-on practice
4. **Mock Interviews**: Use mixed files for comprehensive review

### File Usage Patterns
- **Morning Review**: Quick scan of all question headers
- **Deep Study**: Focus on one technology file per session
- **Pre-Interview**: Review Unclassified file for mixed practice
- **Post-Interview**: Add new questions encountered to appropriate files

### Content Maintenance
- **Regular Updates**: Add new questions after each interview or study session
- **Difficulty Tagging**: Mark questions with experience level requirements
- **Answer Quality**: Include both brief and detailed explanations
- **Real Examples**: Add actual interview questions and company-specific patterns