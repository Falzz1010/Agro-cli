# 🤝 Contributing to AgroCLI

Thank you for your interest in contributing to AgroCLI! This document provides guidelines for contributing to the project.

## 🌟 Ways to Contribute

- 🐛 Report bugs
- 💡 Suggest new features
- 📝 Improve documentation
- 🔧 Submit bug fixes
- ✨ Add new features
- 🧪 Write tests
- 🌍 Translate to other languages

## 🚀 Getting Started

### 1. Fork the Repository
```bash
# Fork on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/agrocli.git
cd agrocli
```

### 2. Set Up Development Environment
```bash
# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt

# Install development dependencies
pip install pytest black flake8 mypy
```

### 3. Create a Branch
```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

## 📋 Development Guidelines

### Code Style

We follow PEP 8 with some modifications:

```python
# Good
def water_plant(plant_name: str, duration: int = 3) -> bool:
    """Water a specific plant for given duration"""
    pass

# Bad
def waterPlant(plantName,duration=3):
    pass
```

**Run linters:**
```bash
black .  # Format code
flake8 .  # Check style
mypy .  # Type checking
```

### Commit Messages

Follow conventional commits:

```
feat: add AI agent mode
fix: resolve WebSocket timeout issue
docs: update API documentation
test: add unit tests for database module
refactor: optimize sensor reading logic
```

### Testing

Write tests for new features:

```python
# tests/test_database.py
def test_add_plant():
    result = add_plant("tomato", "Test-Plant")
    assert result == True
```

Run tests:
```bash
pytest
pytest --cov  # With coverage
```

### Documentation

- Update README.md for user-facing changes
- Update API_DOCUMENTATION.md for API changes
- Add docstrings to all functions
- Update CHANGELOG.md

## 🔄 Pull Request Process

### 1. Before Submitting

- [ ] Code follows style guidelines
- [ ] All tests pass
- [ ] Documentation is updated
- [ ] Commit messages are clear
- [ ] No merge conflicts

### 2. Submit PR

1. Push to your fork
2. Create Pull Request on GitHub
3. Fill in PR template
4. Link related issues

### 3. PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
How was this tested?

## Checklist
- [ ] Code follows style guidelines
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] No breaking changes (or documented)
```

## 🐛 Bug Reports

Use GitHub Issues with this template:

```markdown
**Describe the bug**
Clear description of the bug

**To Reproduce**
Steps to reproduce:
1. Run command '...'
2. Click on '...'
3. See error

**Expected behavior**
What should happen

**Screenshots**
If applicable

**Environment:**
- OS: [e.g. Windows 11]
- Python version: [e.g. 3.11]
- AgroCLI version: [e.g. 1.1.0]

**Additional context**
Any other information
```

## 💡 Feature Requests

Use GitHub Issues with this template:

```markdown
**Is your feature request related to a problem?**
Description of the problem

**Describe the solution you'd like**
Clear description of desired feature

**Describe alternatives you've considered**
Other solutions you've thought about

**Additional context**
Mockups, examples, etc.
```

## 🏗️ Project Structure

```
agrocli/
├── core/           # Core business logic
│   ├── database.py     # Database operations
│   ├── engine.py       # Task calculation
│   ├── realtime.py     # WebSocket manager
│   ├── ai_agent.py     # AI command parsing
│   └── ai_executor.py  # AI action execution
├── hardware/       # Hardware abstraction
│   ├── sensors.py      # Sensor reading
│   └── pump.py         # Pump control
├── web/            # Web server
│   └── server.py       # FastAPI application
├── tests/          # Unit tests (to be added)
├── docs/           # Additional documentation
└── main.py         # CLI entry point
```

## 🎯 Priority Areas

We especially welcome contributions in:

1. **Testing** - Unit tests, integration tests
2. **Hardware Integration** - Real sensor support
3. **AI Features** - LLM integration, better NLP
4. **Mobile App** - React Native or Flutter
5. **Internationalization** - More languages
6. **Performance** - Optimization, caching
7. **Security** - Authentication, authorization

## 📞 Communication

- **GitHub Issues** - Bug reports, feature requests
- **GitHub Discussions** - Questions, ideas
- **Pull Requests** - Code contributions

## 📜 Code of Conduct

### Our Pledge

We pledge to make participation in our project a harassment-free experience for everyone.

### Our Standards

**Positive behavior:**
- Using welcoming language
- Being respectful of differing viewpoints
- Gracefully accepting constructive criticism
- Focusing on what is best for the community

**Unacceptable behavior:**
- Trolling, insulting/derogatory comments
- Public or private harassment
- Publishing others' private information
- Other unethical or unprofessional conduct

## 🙏 Recognition

Contributors will be:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Given credit in documentation

## ❓ Questions?

Feel free to:
- Open a GitHub Discussion
- Comment on related issues
- Reach out to maintainers

---

**Thank you for contributing to AgroCLI! 🌱**
