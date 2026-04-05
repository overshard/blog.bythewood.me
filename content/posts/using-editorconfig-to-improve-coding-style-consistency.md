---
title: Using EditorConfig to improve coding style consistency
slug: using-editorconfig-to-improve-coding-style-consistency
date: 2022-05-21
publish_date: 2022-05-21
tags: coding
description: EditorConfig has been around for almost a decade at this point. It is widely supported by many editors natively and many more with plugins.
cover_image: editorconfig.org.webp
---

EditorConfig has been around for almost a decade at this point. It is widely supported by many editors natively and many more with plugins. You can find more information about their support editors on [EditorConfig's site](https://editorconfig.org/). My current configuration looks something like this.

```shell
root = true

[*]
charset = utf-8
indent_style = space
indent_size = 2
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true

[*.py]
indent_size = 4

[*.md]
trim_trailing_whitespace = false
indent_size = 4
```

I currently use a standard `.editorconfig` on all of my projects and the only exceptions are Python and Markdown since I prefer an indent size of 2 but Python's community has standardized around 4 spaces. Markdown also has an odd way of adding line breaks by adding a space at the end of a line so I have to avoid stripping that extra whitespace.
