# QI text format

Quick Information Compiler - compiler for .qi binary config files, used by FarOS

## Datatypes

| Datatype | Meaning |
| - | - |
| `int8` - `int64` by mult. of 8 | raw integer values (a word of that width) of respective bit size |
| `block`/`endblock` | block of several entries (no resulting data change) |
| `rept`/`endrept` | repeat the contents a provided number of times. data inside can still be accessed elsewhere |

## Structure

- `datatype`: `/([iu]\d{1,2}|(end)?(block|rept))/`
- `label`: `[a-z]+`
- `combination`: `\[value [+-*/] value\]`
- `value`: `integer | hexinteger | =label`
- `entry`: `datatype label : value`

Only entries inside blocks are included in final output - intermediate entries are allowed outside of blocks, as only one modification is allowed per each expression

## Special Compiler Commands

- `!magic` and `!nomagic` - enables/disables magic number, default on
- `!align [value]` - aligns file to certain binary block width
- `e$[VAR]` - get environment variable
