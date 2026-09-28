import sys
import os
import re

def archive_task(task_pattern, source_path, archive_path):
    if not os.path.exists(source_path):
        print(f"Error: {source_path} not found")
        return False
    
    with open(source_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Try to find a section starting with ## and containing the task_pattern
    # but ending before the next ## section.
    # We look for something like '## [Status] [Name] [task_pattern]'
    sections = re.split(r'(\n## )', content)
    
    new_active_sections = []
    archived_blocks = []
    
    # The first split might not be a section
    header = sections[0]
    new_active_sections.append(header)
    
    found = False
    for i in range(1, len(sections), 2):
        marker = sections[i]
        section_text = sections[i+1]
        full_section = marker + section_text
        
        if re.search(task_pattern, full_section, re.IGNORECASE):
            print(f"Archiving section: {full_section.splitlines()[0]}")
            archived_blocks.append(full_section)
            found = True
        else:
            new_active_sections.append(full_section)
    
    if not found:
        print(f"Warning: Task pattern '{task_pattern}' not found in any section.")
        return False
    
    # Update active file
    with open(source_path, 'w', encoding='utf-8') as f:
        f.write("".join(new_active_sections))
    
    # Append to archive
    with open(archive_path, 'a', encoding='utf-8') as f:
        f.write("\n\n" + "".join(archived_blocks))
    
    print("Archival completed successfully")
    return True

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python archive_task.py <task_pattern>")
        sys.exit(1)
    
    pattern = sys.argv[1]
    source = r'c:\Users\kanal\Documents\SCMessenger\SCMessenger\REMAINING_WORK_TRACKING.md'
    archive = r'c:\Users\kanal\Documents\SCMessenger\SCMessenger\docs\ARCHIVE_WORK_TRACKING.md'
    
    archive_task(pattern, source, archive)
