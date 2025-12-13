# Core Markdown Test File

This file tests CommonMark compliance and GitHub Flavored Markdown (GFM) extensions.

---

## Headings

# Heading Level 1
## Heading Level 2
### Heading Level 3
#### Heading Level 4
##### Heading Level 5
###### Heading Level 6

Setext Style Heading 1
======================

Setext Style Heading 2
----------------------

### Headings with Closing Hashes ###

### Heading with *emphasis* and `code`

### Heading with [link](http://example.com)

####### Not a heading (too many hashes)

#Not a heading (no space)

## Custom Heading IDs {#custom-id}

## Another Custom ID {#another-id .class-name}

---

## Paragraphs and Line Breaks

This is a paragraph with
a soft line break (just newline).

This is a paragraph with  
a hard line break (two trailing spaces).

This is a paragraph with\
a hard line break (backslash).

This is one paragraph.

This is another paragraph after a blank line.

This is a very long paragraph that goes on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on to test line wrapping behavior.

---

## Emphasis

*italic with asterisks*

_italic with underscores_

**bold with asterisks**

__bold with underscores__

***bold and italic with asterisks***

___bold and italic with underscores___

**_bold and italic mixed_**

*__bold and italic mixed other way__*

This is *italic in the* middle of a sentence.

This is **bold in the** middle of a sentence.

This_is_not_italic_because_underscores_in_words

This*is*italic*though

Escaped \*asterisks\* and \_underscores\_

**bold with *nested italic* inside**

*italic with **nested bold** inside*

---

## Strikethrough (GFM)

~~This text is struck through~~

~~Strikethrough with **bold** inside~~

This is ~~not struck~~ through properly if tildes aren't doubled

~~Multi
line
strikethrough~~

---

## Blockquotes

> This is a blockquote.

> This is a blockquote
> that spans multiple lines.

> This is a blockquote
with a lazy continuation line.

> Level 1
>> Level 2
>>> Level 3
>>>> Level 4

> Blockquote with **bold** and *italic* and `code`.

> Blockquote with a list:
> - Item 1
> - Item 2
> - Item 3

> Blockquote with code block:
> ```
> code here
> ```

> First paragraph in blockquote.
>
> Second paragraph in blockquote.

---

## Lists

### Unordered Lists

- Item with dash
- Item with dash

* Item with asterisk
* Item with asterisk

+ Item with plus
+ Item with plus

- Item 1
- Item 2
- Item 3

### Ordered Lists

1. First item
2. Second item
3. Third item

1. First item
1. Second item
1. Third item

1. First item
3. Second item (number doesn't matter after first)
8. Third item

0. Starting with zero
1. Next item

### Starting at Different Numbers

5. Starting at five
6. Six
7. Seven

### Nested Lists

- Level 1
  - Level 2
    - Level 3
      - Level 4

1. Level 1
   1. Level 2
      1. Level 3
         1. Level 4

- Unordered parent
  1. Ordered child
  2. Ordered child
- Unordered parent again

1. Ordered parent
   - Unordered child
   - Unordered child
2. Ordered parent again

### Loose vs Tight Lists

Tight list:
- Item 1
- Item 2
- Item 3

Loose list:
- Item 1

- Item 2

- Item 3

### Lists with Multiple Paragraphs

- First paragraph of item.

  Second paragraph of item.

  Third paragraph of item.

- Next item.

### Lists with Blockquotes

- Item with blockquote:

  > This is a blockquote inside a list item.

- Next item.

### Lists with Code Blocks

- Item with code:

  ```python
  def hello():
      print("Hello from a list!")
  ```

- Item with indented code:

      indented code block in list
      needs extra indentation

- Next item.

### Task Lists (GFM)

- [ ] Unchecked task
- [x] Checked task
- [X] Also checked (capital X)
- [ ] Another unchecked

Nested task list:
- [ ] Parent task
  - [ ] Child task 1
  - [x] Child task 2
- [x] Another parent

### Edge Cases

-Not a list (no space)

-  Two spaces after dash
-   Three spaces

1.Not a list (no space)

- List item with trailing spaces   

---

## Links

### Inline Links

[Basic link](http://example.com)

[Link with title](http://example.com "Title here")

[Link with title single quotes](http://example.com 'Title here')

[Link with title parentheses](http://example.com (Title here))

[Empty link]()

[Link with space in URL](<http://example.com/path with spaces>)

### Reference Links

[Reference link][ref1]

[Reference link with space] [ref2]

[Implicit reference link][]

[Implicit reference link]

[ref1]: http://example.com
[ref2]: http://example.com/2 "Title for ref2"
[Implicit reference link]: http://example.com/implicit

[Case insensitive][REF1]

### Autolinks

<http://example.com>

<https://example.com>

<mailto:user@example.com>

<user@example.com>

### GFM Autolinks

http://example.com

https://example.com

www.example.com

user@example.com (email autolink)

### Relative Links

[Relative link](./other-file.md)

[Parent directory](../parent/file.md)

[Anchor link](#headings)

[Anchor to custom ID](#custom-id)

### Links with Special Characters

[Link with parentheses](http://example.com/path_(with)_parens)

[Link with escaped parens](http://example.com/path_\(escaped\))

[Link with query](http://example.com?foo=bar&baz=qux)

[Link with fragment](http://example.com#section)

[Link with port](http://example.com:8080/path)

### Nested Formatting in Links

[**Bold link**](http://example.com)

[*Italic link*](http://example.com)

[`Code link`](http://example.com)

[Link with ![image](http://example.com/img.png)](http://example.com)

---

## Images

### Inline Images

![Alt text](http://example.com/image.png)

![Alt text](http://example.com/image.png "Image title")

![](http://example.com/no-alt.png)

![Alt with special "characters" & stuff](http://example.com/image.png)

### Reference Images

![Reference image][img1]

![Implicit reference image][]

[img1]: http://example.com/ref-image.png "Reference image title"
[Implicit reference image]: http://example.com/implicit-image.png

### Image Dimensions (extended syntax)

![Alt text](http://example.com/image.png){width=100 height=50}

![Alt text](http://example.com/image.png){width=50%}

### Linked Images

[![Alt text](http://example.com/image.png)](http://example.com)

[![Alt text](http://example.com/image.png "Image title")](http://example.com "Link title")

---

## Code

### Inline Code

This is `inline code` in a sentence.

Use `backticks` for code.

Escape backticks: `` `backtick` ``

Double backticks: `` `code with `backticks` inside` ``

Code with ``literal `backtick` inside``

`code with trailing space `

` code with leading space`

`   code with multiple spaces   `

### Indented Code Blocks

    This is an indented code block.
    It continues on multiple lines.
    
    Even with blank lines inside.

    function example() {
        return "indented";
    }

### Fenced Code Blocks

```
Plain fenced code block
No language specified
```

```plaintext
Plaintext code block
```

~~~
Fenced with tildes
~~~

````
Fenced with four backticks
Can contain ``` inside
````

### Fenced Code with Languages

```javascript
function hello() {
    console.log("Hello, World!");
}
```

```python
def hello():
    print("Hello, World!")

class Example:
    def __init__(self):
        self.value = 42
```

```rust
fn main() {
    println!("Hello, World!");
}
```

```go
package main

import "fmt"

func main() {
    fmt.Println("Hello, World!")
}
```

```java
public class Hello {
    public static void main(String[] args) {
        System.out.println("Hello, World!");
    }
}
```

```c
#include <stdio.h>

int main() {
    printf("Hello, World!\n");
    return 0;
}
```

```cpp
#include <iostream>

int main() {
    std::cout << "Hello, World!" << std::endl;
    return 0;
}
```

```csharp
using System;

class Program {
    static void Main() {
        Console.WriteLine("Hello, World!");
    }
}
```

```ruby
puts "Hello, World!"
```

```php
<?php
echo "Hello, World!";
?>
```

```swift
print("Hello, World!")
```

```kotlin
fun main() {
    println("Hello, World!")
}
```

```scala
object Hello extends App {
  println("Hello, World!")
}
```

```typescript
function hello(): void {
    console.log("Hello, World!");
}
```

```html
<!DOCTYPE html>
<html>
<head>
    <title>Hello</title>
</head>
<body>
    <h1>Hello, World!</h1>
</body>
</html>
```

```css
body {
    font-family: sans-serif;
    color: #333;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
}
```

```scss
$primary-color: #333;

body {
    font-family: sans-serif;
    color: $primary-color;
    
    .container {
        max-width: 1200px;
    }
}
```

```json
{
    "name": "example",
    "version": "1.0.0",
    "dependencies": {
        "foo": "^1.0.0"
    }
}
```

```yaml
name: example
version: 1.0.0
dependencies:
  - foo
  - bar
config:
  enabled: true
```

```toml
[package]
name = "example"
version = "1.0.0"

[dependencies]
foo = "1.0"
```

```xml
<?xml version="1.0" encoding="UTF-8"?>
<root>
    <element attribute="value">Content</element>
</root>
```

```sql
SELECT id, name, email
FROM users
WHERE active = true
ORDER BY created_at DESC
LIMIT 10;
```

```bash
#!/bin/bash
echo "Hello, World!"
for i in {1..5}; do
    echo "Iteration $i"
done
```

```shell
$ echo "Hello"
Hello
$ ls -la
total 0
drwxr-xr-x  2 user user  40 Jan  1 00:00 .
```

```powershell
Write-Host "Hello, World!"
Get-ChildItem | Where-Object { $_.Length -gt 1MB }
```

```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
EXPOSE 3000
CMD ["npm", "start"]
```

```makefile
CC = gcc
CFLAGS = -Wall -g

all: main

main: main.o utils.o
	$(CC) $(CFLAGS) -o main main.o utils.o

clean:
	rm -f *.o main
```

```diff
--- a/file.txt
+++ b/file.txt
@@ -1,3 +1,4 @@
 unchanged line
-removed line
+added line
 another unchanged line
+new line at end
```

```markdown
# Markdown in code block

- List item
- **Bold**

[Link](http://example.com)
```

```latex
\documentclass{article}
\begin{document}
Hello, \LaTeX!
\end{document}
```

```r
# R code
data <- c(1, 2, 3, 4, 5)
mean(data)
plot(data)
```

```matlab
% MATLAB code
x = 0:0.1:2*pi;
y = sin(x);
plot(x, y);
```

```haskell
main :: IO ()
main = putStrLn "Hello, World!"

factorial :: Integer -> Integer
factorial 0 = 1
factorial n = n * factorial (n - 1)
```

```elixir
defmodule Hello do
  def world do
    IO.puts("Hello, World!")
  end
end
```

```clojure
(defn hello []
  (println "Hello, World!"))

(hello)
```

```lua
print("Hello, World!")

function factorial(n)
    if n == 0 then
        return 1
    else
        return n * factorial(n - 1)
    end
end
```

```perl
#!/usr/bin/perl
use strict;
use warnings;

print "Hello, World!\n";
```

```awk
BEGIN { print "Hello, World!" }
{ print $0 }
END { print "Done" }
```

```vim
" Vim script
set number
set tabstop=4
nnoremap <leader>w :w<CR>
```

```nginx
server {
    listen 80;
    server_name example.com;
    
    location / {
        proxy_pass http://localhost:3000;
    }
}
```

```apache
<VirtualHost *:80>
    ServerName example.com
    DocumentRoot /var/www/html
    
    <Directory /var/www/html>
        AllowOverride All
    </Directory>
</VirtualHost>
```

```ini
[section]
key = value
another_key = another_value

[another_section]
foo = bar
```

```graphql
type Query {
    user(id: ID!): User
    users: [User!]!
}

type User {
    id: ID!
    name: String!
    email: String!
}
```

```protobuf
syntax = "proto3";

message User {
    int32 id = 1;
    string name = 2;
    string email = 3;
}
```

```terraform
resource "aws_instance" "example" {
  ami           = "ami-0c55b159cbfafe1f0"
  instance_type = "t2.micro"
  
  tags = {
    Name = "example-instance"
  }
}
```

```nix
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.nodejs
    pkgs.yarn
  ];
}
```

### Code Block with Line Numbers (extended)

```python {linenos=true}
def hello():
    print("Hello")

def world():
    print("World")
```

### Code Block with Highlighted Lines (extended)

```python {hl_lines=[2,4]}
def example():
    important_line()
    normal_line()
    another_important()
```

### Code Block with Filename (extended)

```python title="example.py"
def hello():
    print("Hello")
```

---

## Tables (GFM)

### Basic Table

| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |

### Alignment

| Left | Center | Right |
|:-----|:------:|------:|
| L    | C      | R     |
| 1    | 2      | 3     |

### Minimal Syntax

| H1 | H2 |
|-|-|
| A | B |

### No Leading/Trailing Pipes

H1 | H2 | H3
---|---|---
A | B | C

### Formatted Content in Tables

| Feature | Example | Notes |
|---------|---------|-------|
| **Bold** | `code` | *italic* |
| [Link](http://example.com) | ~~strike~~ | Normal |
| Inline $math$ | ![img](i.png) | Mixed |

### Wide Table

| Column 1 | Column 2 | Column 3 | Column 4 | Column 5 | Column 6 | Column 7 | Column 8 |
|----------|----------|----------|----------|----------|----------|----------|----------|
| Data | Data | Data | Data | Data | Data | Data | Data |

### Table with Long Content

| Short | Very Long Content That Might Wrap |
|-------|-----------------------------------|
| A | This is a very long cell that contains a lot of text and might need to wrap or scroll horizontally depending on the renderer |

### Escaped Pipes

| Header | With \| Pipe |
|--------|--------------|
| Cell   | Also \| pipe |

### Empty Cells

| A | B | C |
|---|---|---|
|   | B |   |
| A |   | C |

---

## Horizontal Rules

Three or more hyphens:

---

Three or more asterisks:

***

Three or more underscores:

___

With spaces:

- - -

* * *

_ _ _

Many characters:

--------------------------------------------------

---

## HTML (Raw)

### Inline HTML

This is <strong>bold</strong> and <em>italic</em>.

<span style="color: red;">Red text</span>

<abbr title="Hypertext Markup Language">HTML</abbr>

### Block HTML

<div>
This is a div block.
</div>

<p>This is a paragraph element.</p>

<details>
<summary>Click to expand</summary>

Hidden content here.

- Can include markdown
- Inside HTML blocks

</details>

<table>
<tr>
<td>HTML</td>
<td>Table</td>
</tr>
</table>

<!-- This is an HTML comment -->

<!--
Multi-line
HTML comment
-->

<pre>
Preformatted
    text
        here
</pre>

<kbd>Ctrl</kbd> + <kbd>C</kbd>

<mark>Highlighted text</mark>

Text with <sub>subscript</sub> and <sup>superscript</sup>

<dl>
<dt>Term</dt>
<dd>Definition</dd>
</dl>

---

## Footnotes (Extended)

Here is a sentence with a footnote.[^1]

Another sentence with a different footnote.[^2]

Use named footnotes for clarity.[^note]

Inline footnote.^[This is the footnote content inline.]

[^1]: This is the first footnote.

[^2]: This is the second footnote.
    
    It can have multiple paragraphs.
    
    And even code:
    
    ```
    code in footnote
    ```

[^note]: This is a named footnote.

---

## Definition Lists (Extended)

Term 1
: Definition 1a
: Definition 1b

Term 2
: Definition 2

Term with *formatting*
: Definition with **bold** and `code`

---

## Abbreviations (Extended)

The HTML specification is maintained by the W3C.

*[HTML]: Hypertext Markup Language
*[W3C]: World Wide Web Consortium

---

## Superscript and Subscript (Extended)

Superscript: x^2^ or 2^10^

Subscript: H~2~O or x~i~

Combined: x~i~^2^

---

## Highlight/Mark (Extended)

This is ==highlighted text== in a sentence.

==Multiple words highlighted==

==Highlight with **bold** inside==

---

## Emoji (Extended)

### Shortcodes

:smile: :heart: :thumbsup: :rocket: :star:

:warning: :information_source: :bulb: :memo:

:white_check_mark: :x: :question: :exclamation:

### Unicode Emoji

😀 ❤️ 👍 🚀 ⭐

### Emoji in Context

I :heart: markdown! 🎉

---

## Smart Typography (Extended)

### Quotes

"Double quotes" become smart quotes.

'Single quotes' too.

"Nested 'quotes' work" as well.

### Dashes

En-dash: 1--10 or pages 5--20

Em-dash: Wait---what?

### Ellipsis

Wait for it...

### Fractions

1/2, 1/4, 3/4

### Other

(c) (r) (tm)

+- and -+

---

## Escaping

\*Not italic\*

\**Not bold\**

\# Not a heading

\- Not a list

\> Not a blockquote

\`Not code\`

\[Not a link\](http://example.com)

\![Not an image\](http://example.com/img.png)

\\Backslash

\| Not a table pipe

Backslash at end of line\
(hard break)

---

## Edge Cases and Stress Tests

### Nested Structures

> Blockquote with list:
> - Item 1
>   > Nested blockquote
>   > - Nested list in nested blockquote
> - Item 2

### Deep Nesting

- Level 1
  - Level 2
    - Level 3
      - Level 4
        - Level 5
          - Level 6
            - Level 7
              - Level 8

### Adjacent Elements

**Bold***italic*

*italic***bold**

`code`**bold**

**bold**`code`

[link](http://a.com)[another](http://b.com)

### Unicode and Special Characters

Ümläuts and àccénts

中文字符

日本語

한국어

العربية

עברית

Ελληνικά

Кириллица

### Zero-Width Characters

Zero​Width​Joiner (ZWJ between words)

### Long Unbroken Strings

Thisisaverylongstringwithnospacesthatmightcauseoverflowissuesinnarrowcontainers

`thisisaverylongcodestringwithnospacesthatmightcauseoverflowissues`

### Empty Elements

****

____

``

[]()

![]()

### Whitespace Variations

	Tab-indented text

   Three-space indent

    Four-space indent (code)

Text with     multiple     spaces.

---

## End of Core Markdown Test File
