1. Download the [Highlight](https://open-vsx.org/extension/fabiospampinato/vscode-highlight) plugin
2. Apply the following patterns in `settings.json`. I recommend removing the default highlights since they do not seem to work well (example: `// todo!()` in rust code gets recognized as "todo marker")
```json
{
    "highlight.regexes": {
        "([a-z, A-Z, _][a-z, A-Z, 0-9, _]*?)(:)": {
            "regexFlags": "g",
            "filterFileRegex": ".*\\.casm",
            "decorations": [
                { "color": "#cc00cc" },
                { "color": "#cc00cc" } 
            ]
        },
        "(//(.*))": {
            "regexFlags": "g",
            "filterFileRegex": ".*\\.casm",
            "decorations": [
                { "color": "#777777" },
            ]
        },
        "(@(.*))": {
            "regexFlags": "g",
            "filterFileRegex": ".*\\.casm",
            "decorations": [
                { "color": "#f06c00" },
            ]
        },
        "(%[a-z, 0-9]{1,2}\\s)": {
            "regexFlags": "g",
            "filterFileRegex": ".*\\.casm",
            "decorations": [
                { "color": "#f0cc00" },
            ]
        },
        "(\\.[a-z]*)": {
            "regexFlags": "g",
            "filterFileRegex": ".*\\.casm",
            "decorations": [
                { "color": "#ff009d" },
            ]
        },
        "(\".*?\")": {
            "regexFlags": "g",
            "filterFileRegex": ".*\\.casm",
            "decorations": [
                { "color": "#2a9600" },
            ]
        },
        "(\\b((0[bqozzxd])|[0-9])[0-9,a-f,A-F]*(\\.)?[0-9,a-f,A-F]*(i)*)": {
            "regexFlags": "g",
            "filterFileRegex": ".*\\.casm",
            "decorations": [
                { "color": "#0890ff" },
            ]
        },
    }
}
```