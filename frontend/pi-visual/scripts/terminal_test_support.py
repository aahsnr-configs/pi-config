"""Small ANSI screen projection for assertions on our Ratatui PTY output.

Not a Kitty emulator: this only interprets cursor/erase operations used by the
cell frontend. Real terminal graphics/keyboard/IME acceptance stays separate.
"""
import re
import unicodedata

def screen_text(output, columns=100, rows=24):
    cells=[[' ']*columns for _ in range(rows)]
    row=col=0
    text=output.decode('utf-8',errors='replace')
    index=0
    while index<len(text):
        if text[index]=='\x1b':
            match=re.match(r'\x1b\[([0-9;?<>]*)([@-~])',text[index:])
            if match:
                params,code=match.groups()
                index+=len(match[0])
                if not params or params[0].isdigit():
                    values=[int(p or 0) for p in params.split(';')]
                    if code in ('H','f'):
                        row=max(0,(values[0] or 1)-1)
                        col=max(0,((values[1] if len(values)>1 else 1) or 1)-1)
                    elif code=='A':row=max(0,row-(values[0] or 1))
                    elif code=='B':row+=values[0] or 1
                    elif code=='C':col+=values[0] or 1
                    elif code=='D':col=max(0,col-(values[0] or 1))
                    elif code=='G':col=max(0,(values[0] or 1)-1)
                    elif code=='J' and values[0] in (2,3):cells=[[' ']*columns for _ in range(rows)]
                    elif code=='K' and row<rows:
                        for x in range(0 if values[0] in (1,2) else min(col,columns),min(col+1,columns) if values[0]==1 else columns):cells[row][x]=' '
                continue
            index+=1
            continue
        char=text[index]
        index+=1
        if char=='\r':col=0
        elif char=='\n':row+=1
        elif char>=' ' and not unicodedata.combining(char):
            if col>=columns:col=0;row+=1
            if row<rows:cells[row][col]=char
            col+=2 if unicodedata.east_asian_width(char) in ('W','F') else 1
    return '\n'.join(''.join(line) for line in cells)
