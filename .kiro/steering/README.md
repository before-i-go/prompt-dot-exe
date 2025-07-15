# Kiro AI Steering System - Interview Preparation Guidance

This directory contains steering documents that guide the Kiro AI assistant in providing contextually appropriate help for backend engineering interview preparation. These documents ensure consistent, focused assistance aligned with your learning goals.

## 🎯 Purpose and Philosophy

### Steering System Overview
The steering system provides contextual guidance to AI assistants, ensuring they understand:
- **Your Learning Objectives**: Backend engineering interview preparation
- **Technology Focus**: Rust-first development with multi-language competency
- **Skill Level Targeting**: Junior to Senior backend engineering roles
- **Interview Context**: Technical, behavioral, and system design preparation

### Why Steering Matters
- **Consistent Guidance**: AI responses align with your preparation goals
- **Context Awareness**: Assistant understands the interview preparation context
- **Focused Learning**: Avoids generic advice, provides targeted technical guidance
- **Progressive Difficulty**: Adapts recommendations based on experience level

## 📁 Steering Document Structure

```
.kiro/steering/
├── README.md           # This overview document
├── product.md          # Product vision and target audience
├── tech.md             # Technology stack and interview categories
└── structure.md        # Repository organization principles
```

## 📋 Document Breakdown

### product.md - Strategic Focus
**Purpose**: Defines the overall product vision and target audience

**Key Elements**:
- **Core Focus Areas**: Interview question banks, concept learning, system design
- **Target Audience**: Backend engineers preparing for FAANG and high-growth startups
- **Value Propositions**: Question-driven learning, multi-language coverage, progressive difficulty
- **Success Metrics**: Technical competency indicators and interview performance goals

**AI Guidance**: Helps the assistant understand you're preparing for senior-level backend roles, not general programming help.

### tech.md - Technical Competencies
**Purpose**: Outlines technology categories and interview question types

**Key Elements**:
- **Programming Languages**: Python, Go, Rust, JavaScript/Node.js, Java, Ruby
- **Frameworks & Tools**: Rails, Django, Express, FastAPI, databases, message queues
- **Interview Categories**: Coding/algorithms, system design, technology deep dives
- **Study Commands**: Practical grep patterns and search strategies

**AI Guidance**: Ensures technical discussions focus on backend-relevant technologies and interview-appropriate depth.

### structure.md - Organization Principles
**Purpose**: Explains repository organization and file relationships

**Key Elements**:
- **Flat Structure Benefits**: Quick access, search-friendly, session-based learning
- **File Naming Conventions**: Date-based organization for tracking progress
- **Content Categories**: Interactive materials, research documents, interview materials
- **Workflow Patterns**: Daily prep routines, study session organization

**AI Guidance**: Helps the assistant navigate and recommend appropriate files based on your current learning needs.

## 🤖 How Kiro Uses These Documents

### Context-Aware Responses
When you ask questions, Kiro considers:
- **Your Current Focus**: Are you studying algorithms, system design, or specific technologies?
- **Experience Level**: Adjusts complexity based on junior/mid/senior targeting
- **Interview Context**: Frames answers in terms of what interviewers expect
- **Practical Application**: Connects theory to real-world backend scenarios

### Example Interactions

**Without Steering**:
```
You: "How do I optimize database queries?"
AI: "Here are general database optimization techniques..."
```

**With Steering**:
```
You: "How do I optimize database queries?"
AI: "For backend engineering interviews, focus on these key optimization strategies that demonstrate senior-level thinking:

1. Indexing strategies (B-tree vs hash indexes)
2. Query plan analysis and EXPLAIN usage
3. N+1 query problems and solutions
4. Connection pooling and prepared statements

This connects to system design questions about scaling databases and is often asked in the context of Rails/Django ORM optimization..."
```

### Technology Recommendations
Based on the steering documents, Kiro will:
- **Prioritize Rust**: For systems programming and performance discussions
- **Include Multiple Languages**: Show polyglot competency expected at senior levels
- **Focus on Backend**: Avoid frontend-heavy recommendations unless specifically requested
- **Interview Relevance**: Connect technical concepts to common interview scenarios

## 🔧 Customizing Steering Documents

### Adding New Focus Areas
To expand the steering system:

1. **Identify Gaps**: What interview topics aren't well covered?
2. **Create Focused Documents**: Add new `.md` files for specific domains
3. **Update References**: Cross-link between related steering documents
4. **Test Guidance**: Verify AI responses align with new guidance

### Example: Adding DevOps Focus
```markdown
# devops.md - Infrastructure and Operations Interview Prep

## Core Competencies
- Container orchestration (Docker, Kubernetes)
- CI/CD pipeline design and implementation
- Infrastructure as Code (Terraform, CloudFormation)
- Monitoring and observability (Prometheus, Grafana, ELK stack)

## Interview Question Categories
- "How would you deploy a microservices architecture?"
- "Design a CI/CD pipeline for a multi-environment setup"
- "How do you handle secrets management in production?"
```

### Steering Document Best Practices
- **Specific Over General**: Provide concrete examples and scenarios
- **Interview-Focused**: Frame everything in terms of interview preparation
- **Progressive Complexity**: Include guidance for different experience levels
- **Cross-Referenced**: Link related concepts across documents

## 🎓 Learning Integration

### How Steering Enhances Your Preparation

#### Consistent Messaging
- All AI interactions reinforce your backend engineering focus
- Technical discussions maintain appropriate depth for your target level
- Recommendations align with modern backend development practices

#### Contextual Relevance
- Questions about Rust get systems programming context
- Database discussions include scalability considerations
- Algorithm problems connect to real backend scenarios

#### Interview Simulation
- AI responses mirror what senior engineers expect to hear
- Technical explanations include trade-off analysis
- Solutions consider production deployment concerns

### Measuring Steering Effectiveness

**Good Indicators**:
- AI consistently recommends backend-relevant technologies
- Responses include interview-appropriate depth and context
- Technical discussions connect to real-world scenarios
- Recommendations align with your experience level goals

**Adjustment Signals**:
- AI provides too generic or too advanced guidance
- Recommendations don't align with backend engineering roles
- Missing connections between theory and interview scenarios

## 🚀 Advanced Steering Techniques

### Conditional Inclusion
Some steering documents can be conditionally included based on:
- **File Context**: When working with specific technologies
- **Manual Triggers**: Using context keys in chat
- **Time-Based**: Different guidance for different preparation phases

### Dynamic Context
Steering documents can reference external files:
```markdown
# Include API specifications for system design discussions
#[[file:../api-specs/user-service.yaml]]

# Reference performance benchmarks for optimization talks
#[[file:../benchmarks/database-performance.md]]
```

### Multi-Modal Guidance
Steering works with:
- **Text Conversations**: Direct Q&A and explanations
- **Code Reviews**: Contextual feedback on your implementations
- **File Analysis**: Understanding your codebase and suggesting improvements
- **Image Analysis**: Reviewing system architecture diagrams

## 🤝 Contributing to Steering

### When to Update Steering Documents
- **New Interview Experiences**: Add patterns from recent interviews
- **Technology Evolution**: Update for new frameworks or best practices
- **Skill Development**: Adjust complexity as your expertise grows
- **Gap Identification**: Address areas where AI guidance was insufficient

### Steering Document Maintenance
- **Regular Review**: Quarterly assessment of guidance effectiveness
- **Version Control**: Track changes to understand evolution
- **Testing**: Verify AI responses align with updated guidance
- **Documentation**: Keep this README current with changes

---

**Remember**: Steering documents are your way of teaching the AI assistant how to be most helpful for your specific learning goals. They transform generic AI assistance into focused, contextual guidance tailored for backend engineering interview success.