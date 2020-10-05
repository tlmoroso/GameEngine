from graphviz import render
import sys
from os import path

dot_file = str()

if len(sys.argv) == 1:
    print("ERROR: no path given as argument to script")
else:
    dot_file = sys.argv[1]
    if dot_file == str():
        print("ERROR: path is empty")
    else:
        if path.exists(dot_file):
            render('dot', 'png', sys.argv[1])
        else:
            print("ERROR: path does not exist")
