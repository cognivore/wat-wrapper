sed -E 's/;;.*$//; s/\(;\s*[^)]*;\)//' "$1" > output.wat
