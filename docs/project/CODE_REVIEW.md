# Code Review Guidelines

This document establishes code review standards and processes for the Songbird Orchestrator project to ensure high-quality, maintainable code.

## 🎯 Code Review Objectives

### Primary Goals
- **Quality Assurance**: Catch bugs, performance issues, and design problems
- **Knowledge Sharing**: Distribute domain knowledge across the team
- **Standards Compliance**: Ensure consistent coding standards and patterns
- **Security Review**: Identify potential security vulnerabilities

### Secondary Benefits
- Mentorship and learning opportunities
- Architecture and design validation
- Documentation and test coverage verification

## 📋 Review Process

### 1. Pre-Review Checklist (Author)

Before requesting review, ensure:

- [ ] **Compilation**: Code builds without errors
- [ ] **Tests**: All tests pass (`cargo test`)
- [ ] **Formatting**: Code is formatted (`cargo fmt`)
- [ ] **Linting**: No clippy warnings (`cargo clippy`)
- [ ] **Documentation**: Public APIs are documented
- [ ] **Self-Review**: Author has reviewed their own changes

### 2. Review Request

#### Pull Request Requirements
- **Clear Title**: Descriptive, concise summary
- **Description**: What changes were made and why
- **Context**: Links to issues, design docs, or related PRs
- **Testing**: How changes were tested
- **Breaking Changes**: Highlight any API changes

#### Example PR Template
```markdown
## Summary
Brief description of changes

## Changes Made
- List of specific changes
- Focus on the "what" and "why"

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing completed

## Notes
- Any special considerations
- Breaking changes (if any)
- Follow-up work needed
```

### 3. Review Assignment

#### Automatic Assignment
- Small changes (< 50 lines): 1 reviewer
- Medium changes (50-200 lines): 2 reviewers  
- Large changes (> 200 lines): 2+ reviewers + architecture review

#### Reviewer Selection
- **Domain Expert**: Someone familiar with the affected code
- **Fresh Eyes**: Someone less familiar for different perspective
- **Senior Developer**: For complex architectural changes

### 4. Review Timeline

#### Response Times
- **Initial Response**: Within 24 hours (business days)
- **Full Review**: Within 48 hours for normal changes
- **Urgent Changes**: Same day review with clear justification

#### Review Prioritization
1. **Critical**: Bug fixes, security issues
2. **High**: New features, API changes
3. **Medium**: Refactoring, documentation
4. **Low**: Minor improvements, style changes

## 🔍 Review Focus Areas

### Code Quality

#### Functionality
- **Correctness**: Does the code do what it's supposed to do?
- **Edge Cases**: Are error conditions and edge cases handled?
- **Performance**: Are there obvious performance issues?
- **Resource Management**: Proper memory/resource cleanup?

#### Design & Architecture
- **Single Responsibility**: Each function/struct has a clear purpose
- **Error Handling**: Proper use of Result types and error propagation
- **API Design**: Public interfaces are intuitive and well-designed
- **Patterns**: Consistent with established project patterns

### Rust-Specific Considerations

#### Ownership & Borrowing
- Efficient use of references vs. owned values
- Appropriate lifetimes and lifetime elision
- No unnecessary cloning or allocation

#### Safety & Correctness
- No unsafe code without justification and documentation
- Proper error handling with Result types
- Thread safety considerations for concurrent code

#### Idiomatic Rust
- Use of iterators over manual loops where appropriate
- Proper use of Option and Result types
- Following Rust naming conventions

### Testing & Documentation

#### Test Coverage
- New functionality has appropriate tests
- Tests cover both happy path and error conditions
- Integration tests for complex interactions

#### Documentation
- Public APIs have rustdoc comments
- Complex algorithms are explained
- Examples provided for non-trivial usage

## 💬 Review Communication

### Feedback Categories

#### **Must Fix** (Blocking)
Use for:
- Bugs or correctness issues
- Security vulnerabilities
- API design problems
- Test failures

#### **Should Fix** (Strong Suggestion)
Use for:
- Performance improvements
- Code clarity issues
- Minor design improvements
- Documentation gaps

#### **Consider** (Optional)
Use for:
- Style preferences
- Alternative approaches
- Future optimization opportunities

### Feedback Guidelines

#### Constructive Feedback
- **Be Specific**: Point out exact lines and explain the issue
- **Explain Why**: Provide reasoning behind suggestions
- **Offer Solutions**: Suggest alternatives when pointing out problems
- **Stay Professional**: Focus on the code, not the person

#### Example Comments
```markdown
**Must Fix**: This function can panic if the vector is empty. Consider using 
`vec.first()` or adding a bounds check.

**Should Fix**: This could be more efficient using an iterator chain:
`items.iter().filter(|x| x.is_valid()).collect()`

**Consider**: You might want to extract this logic into a separate function 
for better testability.
```

### Response Guidelines

#### For Authors
- **Address All Feedback**: Respond to every comment
- **Ask Questions**: If feedback is unclear, ask for clarification
- **Push Back Respectfully**: If you disagree, explain your reasoning
- **Update and Re-request**: Make changes and re-request review

#### For Reviewers
- **Timely Reviews**: Review promptly and thoroughly
- **Complete Reviews**: Don't approve prematurely, review all changes
- **Follow Up**: Check that your feedback was addressed
- **Learn and Adapt**: Use reviews as learning opportunities

## ✅ Approval Criteria

### Approval Requirements
- [ ] All "Must Fix" items addressed
- [ ] Tests pass and coverage is adequate
- [ ] Documentation is complete and accurate
- [ ] Code follows project conventions
- [ ] No obvious performance or security issues

### Final Approval
- At least one approval from domain expert
- All requested changes addressed or discussed
- CI/CD pipeline passes completely

## 🚀 Special Review Types

### Security Reviews
Required for:
- Authentication/authorization changes
- External API integrations
- Cryptographic operations
- Input validation and sanitization

### Performance Reviews
Required for:
- Changes to hot paths or critical algorithms
- New dependencies with performance implications
- Memory allocation patterns
- Concurrent/async code changes

### API Reviews
Required for:
- Public API changes or additions
- Breaking changes to existing APIs
- New trait definitions
- Major architectural changes

## 📊 Review Metrics

### Quality Indicators
- Review coverage (% of code reviewed)
- Time to review (average response time)
- Issues found in review vs. production
- Review iteration count

### Continuous Improvement
- Regular retrospectives on review process
- Feedback on review quality and effectiveness
- Updates to guidelines based on team experience

## 🛠️ Tools & Automation

### Automated Checks
- **CI Pipeline**: Automated build, test, and lint checks
- **Code Coverage**: Minimum coverage thresholds
- **Security Scanning**: Automated vulnerability detection
- **Dependency Checks**: License and security validation

### Review Tools
- GitHub PR reviews with inline comments
- Automated reviewer assignment
- Review status tracking
- Integration with issue tracking

---

## 📚 Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Effective Code Reviews](https://google.github.io/eng-practices/review/)
- [The Rust Programming Language](https://doc.rust-lang.org/book/)

**Remember**: Code reviews are collaborative, not adversarial. The goal is to improve code quality and share knowledge, not to find fault. 