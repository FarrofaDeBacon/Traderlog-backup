import sys

file_path = r'C:\PROJETOS\TraderLogPro\2.csv'
try:
    with open(file_path, 'rb') as f:
        content = f.read()
    
    decoded = content.decode('windows-1252')
    lines = decoded.splitlines()
    
    for i, line in enumerate(lines):
        print(f"{i:2}: {line}")
                
except Exception as e:
    print(f"Error: {e}")
