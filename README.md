# Fixed File Highlighter

Highlight parts of a file given a syntax.

We parse over a syntax CSV, expecting a header row containing `start,length,name,condition`, where:

- `start` is the 1-based start column of the character to highlight
- `length` is the number of columns of this field
- `name` is the human readable name for this field
- `condition` (optional) is a regex to restrict this rule applying except to lines that match the regex.

Rules are applied top-to-bottom.

HTML is output to the terminal and can be redirected or copied as desired.

## Usage

```
Usage: fixedfile-highlighter [OPTIONS] <INPUT_FILE> <SYNTAX_FILE>

Arguments:
  <INPUT_FILE>   The input file to process
  <SYNTAX_FILE>  The syntax file to use

Options:
  -c, --colors <COLORS>  The colours to output the analysed file with. This can be one of a number of inputs: a predefined preset (greyscale [default], rainbow) or; a comma separated list of hex codes
  -s, --snippet          Output an HTML snippet, rather than a full file
  -h, --help             Print help (see more with '--help')
  -V, --version          Print version
```

If you wish to save the file, you should redirect the output, as below:

```sh
fixedfile-highlighter inputfile syntax.csv > output.html
```
