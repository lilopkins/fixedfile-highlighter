# Fixed File Highlighter

Highlight parts of a file given a syntax.

We parse over a syntax CSV, expecting a header row containing `start,length,name,condition`, where:

- `start` is the 1-based start column of the character to highlight
- `length` is the number of columns of this field
- `name` is the human readable name for this field
- `condition` (optional) is a regex to restrict this rule applying except to lines that match the regex.

Rules are applied top-to-bottom.

HTML is output to the terminal and can be redirected or copied as desired.
