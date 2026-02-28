#!/usr/bin/env python3
"""Generate changelog for GitHub release."""
import subprocess
import sys
import os

def get_commits(prev_tag=None):
    """Get commits since previous tag."""
    if prev_tag:
        cmd = ['git', 'log', '--pretty=format:- %s (%h)', '--no-merges', f'{prev_tag}..HEAD']
    else:
        cmd = ['git', 'log', '--pretty=format:- %s (%h)', '--no-merges']
    result = subprocess.run(cmd, capture_output=True, text=True)
    return result.stdout

def categorize_commits(commits):
    """Categorize commits by type."""
    features = []
    fixes = []
    docs = []
    refactor = []
    other = []
    
    for line in commits.strip().split('\n'):
        if not line:
            continue
        lower = line.lower()
        if any(x in lower for x in ['feat', 'feature', 'add']):
            features.append(line)
        elif any(x in lower for x in ['fix', 'bugfix', 'hotfix']):
            fixes.append(line)
        elif any(x in lower for x in ['doc', 'docs']):
            docs.append(line)
        elif any(x in lower for x in ['refactor', 'perf']):
            refactor.append(line)
        else:
            other.append(line)
    
    return features, fixes, docs, refactor, other

def generate_changelog(version, prev_tag, repo):
    """Generate changelog body."""
    commits = get_commits(prev_tag if prev_tag else None)
    
    if not commits.strip():
        return f"## What's Changed\n\nNo commits since {prev_tag or 'start'}."
    
    features, fixes, docs, refactor, other = categorize_commits(commits)
    
    body = "## What's Changed\n\n"
    
    if features:
        body += "### 🚀 Features\n" + "\n".join(features) + "\n\n"
    
    if fixes:
        body += "### 🐛 Bug Fixes\n" + "\n".join(fixes) + "\n\n"
    
    if refactor:
        body += "### ⚡ Performance & Refactoring\n" + "\n".join(refactor) + "\n\n"
    
    if docs:
        body += "### 📚 Documentation\n" + "\n".join(docs) + "\n\n"
    
    if other:
        body += "### 🔧 Other Changes\n" + "\n".join(other) + "\n\n"
    
    compare = f"{prev_tag}...{version}" if prev_tag else f"HEAD...{version}"
    body += f"**Full Changelog**: https://github.com/{repo}/compare/{compare}\n"
    
    return body

if __name__ == '__main__':
    version = os.environ.get('VERSION', '')
    repo = os.environ.get('GITHUB_REPOSITORY', '')
    
    # Get previous tag
    result = subprocess.run(['git', 'describe', '--tags', '--abbrev=0', 'HEAD^'], 
                          capture_output=True, text=True)
    prev_tag = result.stdout.strip() if result.returncode == 0 else ''
    
    body = generate_changelog(version, prev_tag, repo)
    
    # Write to file
    with open('RELEASE_NOTES.md', 'w') as f:
        f.write(body)
    
    print(body)
