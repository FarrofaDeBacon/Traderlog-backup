import sys

file_path = r'C:\PROJETOS\TraderLogPro\2.csv'
try:
    with open(file_path, 'rb') as f:
        content = f.read()
    
    # Try decoding with windows-1252 as the rust code does
    decoded = content.decode('windows-1252')
    print(decoded)
except Exception as e:
    print(f"Error: {e}")
