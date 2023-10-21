# Function definition format

#### `Int`|`Str`|`Func`|`Custom`|`Variant` name(\[\[, ] `Int`|`Str`|`Func`|`Custom`|`Variant` argument\_name][[ ]...argument\_list\_name])\[ const|mut Scope]

If function returns nothing, it has nothing before its name.

If function doesn't take arguments, it has empty argument list `<type> name()`.

`const Scope` means this function can access variables and functions directly from scope.

`mut Scope` means this function can change variables and functions.

`<type> ...<argument_list_name>` means this function accepts unlimited number of arguments but only with specific type.

`Variant` means one from all possible types.

If function behaves differently, depending on argument types, another function with same name but different arguments may be listed here.

If function takes at least 1 argument, it should have section `Arguments`, describing what that arguments mean.

If function returns non-`Void` value, it should have `Returns` section, describing what return value of function means.

If function has some additional details, it should have `Notes` section.

If function throw at least 1 exception, that neither argument count nor type mismatch, it should have `Exceptions` section, describing what exception means and when it is thrown.

Level 2 is for module name, level 3 is for submodule name, level 4 is for function name, level 5 is for additional details, level 6 is for details in additional details.

# Functions in Standard Library for Easy Prog

## IO

### StdIO

#### print(`Variant` ...*value*)

Prints to Standard Output (stdout).

##### Arguments

If *value* has type `Void`, prints `<null>`;

if *value* has type `Int`, prints its value as string;

if *value* has type `Str`, prints its value;

If *value* has type `Func`, prints `<function at address $adr>` where `$adr` is address of that function;

If *value* has type `Custom`, prints `<custom type $type at address $adr>` where `$type` is 8-bytes unsigned integer, `$adr` is address of value.

##### Notes

Doesn't output newline. To print newline in the end, add `lf()` as function argument.

#### printerr(`Variant` ...*value*)

Same as `print(Variant ...value)`, but outputs to Standard Error (stderr).

#### Str input()

Gets one line from Standard Input (stdin).

##### Returns

One line from stdin without line feed in the end.

##### Exceptions

###### I/O error

Thrown when failed to read from stdin.

### FileIO

#### `Custom` fopen(`Str` *path*, `Str` *mode*)

Opens file.

##### Arguments

*path* is a path to file, that will be opened;

*mode* is a mode in what file will be opened,
`r` = open for reading, `w` = open for writing.

##### Returns

Pointer to opened file as `Custom`.

##### Exceptions

###### I/O error

Thrown when failed to open file.

##### Notes

Opened file will automatically close in the end of program.
If you want to close file manually, call `fclose(Custom)`.
If you don't want to close file in the end, don't assign it to a variable.

#### `Str` fread(`Custom` *file*)

Reads data from file.

##### Arguments

*file* is a pointer to opened file, returned by `Custom fopen(Str, Str)`.

##### Returns

String with data, read from file.

##### Exceptions

###### I/O error

Thrown when failed to read data from file.

#### fwrite(`Custom` *file*, `Str` *data*)

Writes data to *file*.

##### Arguments

*file* is a pointer to opened file, returned by `Custom fopen(Str, Str)`;

*data* is a string of text/data, that will be written to file.

##### Exceptions

###### I/O error

Thrown when failed to write data to *file*.

#### fclose(`Custom` *file*)

Manually closes specified *file*.

##### Arguments

*file* is a pointer to opened file, returned by `Custom fopen(Str, Str)`;

## String

#### `Int` parse\_int(`Str` *data*)

Parses `Int` from `Str`.

##### Arguments

*data* is string representation of signed 8-byte integer with base of 10.

##### Returns

Parsed to `Int` value.

##### Exceptions

###### Invalid number string

Thrown when failed to parse string (found non-digit character or invalid minus position).

#### `Str` lf()

Returns line feed byte.

##### Returns

`\n`, newline or line feed byte.

#### `Str` cr()

Returns carriage return byte.

##### Returns

`\r` or carriage return byte.

## Core

#### declfunc(`Str` *name*, `Func` *body*) mut Scope

Adds new function to current scope.

##### Arguments

*name* is name for new function;

*body* is body for new function.

#### set(`Str` *name*, `Variant` *value*) mut Scope

Changes value of variable *name* to *value*.

##### Arguments

*name* is name for variable, that may or may not exist in current scope;

*value* is new value for that variable.

#### `Variant` null(`Str` *variable*) mut Scope

Resets *variable* to default.

##### Arguments

*variable* is name of variable to reset.

##### Returns

`Void` if type of *variable* is `Void`;
`0` if type of *variable* is `Int`;
`""` if type of *variable* is `Str`;
`{}` if type of *variable* is `Func`.

##### Exceptions

###### Reset custom error

Thrown, when type of *variable* is `Custom`, because it isn't possible now to reset `Custom` types, using custom reset functions. To reset `Custom`, use specific to that custom type function.

#### if(`Int` *condition*, `Func` *if_branch*)

If *condition* isn't equals to `0`, executes `if_branch`.

##### Arguments

If *condition* is equals to `0`, then does nothing;
Otherwise, executes code in *if_branch*.

#### if\_else(`Int` *condition*, `Func` *if_branch*, `Func` *else_branch*)

if *if_branch* isn't equals to `0`, executes *if_branch*,
else executes *else_branch*.

##### Arguments

If *condition* is equals to `0`, then executes code in *else_branch*;
Otherwise, executes code in *if_branch*.

#### `Int` and(`Int` *a*, `Int` *b)

Adds *a* and *b* together.

##### Arguments

*a* is a first term;
*b* is a second term.

##### Returns

Sum of *a* and *b*.

#### `Int` subt(`Int` *a*, `Int` *b)

Subtracts *b* from *a*.

##### Arguments

*a* is a first term;
*b* is a second term.

##### Returns

Difference of *a* and *b*.

#### `Int` mult(`Int` *a*, `Int` *b)

Multiplies *a* and *b* together.

##### Arguments

*a* is a first factor;
*b* is a second factor.

##### Returns

Product of *a* and *b*.

#### `Int` idiv(`Int` *a*, `Int` *b)

Adds *a* and *b* together.

##### Arguments

*a* is a dividend;
*b* is a divisor.

##### Returns

Fraction of *a* and *b*.

##### Exceptions

###### Division by zero error

Thrown if *b* is `0`.

#### `Int` and(`Int` *condition1*, `Int` *condition2*)

Returns *a* && *b*.

##### Arguments

*condition1* is a first condition;
*condition2* is a second condition.

##### Returns

`1` if *a* != `0` && *b* != `0`,
`0` otherwise.

#### `Int` or(`Int` *condition1*, `Int` *condition2*)

Returns *a* || *b*.

##### Arguments

*condition1* is a first condition;
*condition2* is a second condition.

##### Returns

`1` if *a* != `0` || *b* != `0`,
`0` otherwise.

#### `Int` eq(`Variant` *var1*, `Variant` *var2*)

Returns *a* == *b*.

##### Arguments

*condition1* is a first variable;
*condition2* is a second variable.

##### Returns

`1` if *a* == *b*,
`0` otherwise.

#### `Int` neq(`Variant` *var1*, `Variant` *var2*)

Returns *a* != *b*.

##### Arguments

*condition1* is a first variable;
*condition2* is a second variable.

##### Returns

`1` if *a* != *b*,
`0` otherwise.

#### exit(`Int` *code* = `0`)

Exits with return code *code*.

##### Arguments

*code* is a return code to exit.

##### Notes

Shouldn't be called in the end of program manually.

## Debug

#### inspect\_scope() const Scope

Prints all variables and functions defined in current scope.

# Variables in Standard Library for Easy Prog

## Format

`Int|Str|Func|Custom|Variant` *name* = `<value>`.

## Variables

`Int` *true* = 1;

`Int` *false* = 0.
