import sys

file_path = r'C:\PROJETOS\TraderLogPro\2.csv'
try:
    with open(file_path, 'rb') as f:
        content = f.read()
    
    # Just print the first 1000 bytes as hex to be sure
    print(content.hex(' ', 1))
    
    decoded = content.decode('windows-1252', errors='replace')
    print("\nDECODED:")
    print(decoded)
                
except Exception as e:
    print(f"Error: {e}")
