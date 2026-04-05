import CodeMirror from "codemirror/lib/codemirror.js";

import "codemirror/mode/python/python.js";
import "codemirror/mode/javascript/javascript.js";
import "codemirror/mode/htmlmixed/htmlmixed.js";
import "codemirror/mode/css/css.js";
import "codemirror/mode/shell/shell.js";

import "codemirror/lib/codemirror.css";
import "codemirror/theme/material-darker.css";


document.addEventListener("DOMContentLoaded", () => {
  const blockCode = document.querySelectorAll(".block-code");

  if (blockCode) {
    Array.prototype.forEach.call(blockCode, (block) => {
      const textarea = block.querySelector("textarea");
      CodeMirror.fromTextArea(textarea, {
        theme: "material-darker",
        lineNumbers: true,
        readOnly: true,
        viewportMargin: Infinity,
        mode: textarea.dataset.language,
      });
    });
  }
});
