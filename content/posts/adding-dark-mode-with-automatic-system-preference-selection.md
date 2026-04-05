---
title: Adding dark mode with automatic system preference selection
slug: adding-dark-mode-with-automatic-system-preference-selection
date: 2022-07-02
publish_date: 2022-07-02
tags: webdev
description: Creating a dark, or light, version of your website may seem like a daunting task if you think you need an entirely new color pallet. It's 2022 though and we have the widely supported invert CSS filter.
cover_image: dark-mode-light-mode.webp
---

Luckily all modern [browsers since 2015](https://caniuse.com/?search=invert) have had a CSS filter to help with creating dark and light mode color schemes based on your current theme, invert. With roughly ~95% global support for this filter you can safely create a dark mode version of your site with it. I'm doing it on this site right now!

Here's the dark theme for this website:

```css
/* dark.css */

main.dark {
  background: #e7e7e7;
  filter: invert(1);
}

main.dark img {
  filter: invert(1);
}

main.dark .block-code {
  filter: invert(1);
}

main.dark .reverse-invert {
  filter: invert(1);
}
```

One of the first things you'll want to fix when doing a global invert is images. The invert filter will invert literally everything, including pictures, which you probably don't want inverted. I made a global `img` option to reverse all image inverts by inverting it again. I've seen no noticeable performance loss or image issues from doing this thus far. My blocks of code are already "dark mode" so there's no point in inverting them. There is also a helper class `.reverse-invert` that can be used on the fly when I think it's needed. I also don't invert the entire page and I just invert my content since my navbar and footer work in both dark and light mode. You could easily change this to be `body.dark` to invert everything.

You now have a dark mode theme for your site. Spot check over things and add `.reverse-invert` when you think it's needed and maybe run a lighthouse check for color accessibility doing slight adjustments where required.

The next step is to make our color mode swapper, I add a little bit more CSS for this but mostly relied on Bootstrap classes.

```css
/* dark.css cont. */

#prefers-color-scheme {
  width: 150px;
  background-color: #171a1d;
  border-color: #6b6b6b;
  color: white;
  padding-left: 40px;
}

.prefers-color-scheme-icon {
  position: absolute;
  color: white;
  margin: 8px;
}
```

Then for the HTML element I use a simple select field with some inline SVG icons that I got from [Bootstrap's icon project](https://icons.getbootstrap.com/).

```html
<div id="color-scheme-selector">
  <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" fill="currentColor" class="prefers-color-scheme-icon system" viewBox="0 0 16 16">
    <path d="M13.5 3a.5.5 0 0 1 .5.5V11H2V3.5a.5.5 0 0 1 .5-.5h11zm-11-1A1.5 1.5 0 0 0 1 3.5V12h14V3.5A1.5 1.5 0 0 0 13.5 2h-11zM0 12.5h16a1.5 1.5 0 0 1-1.5 1.5h-13A1.5 1.5 0 0 1 0 12.5z"/>
  </svg>
  <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" fill="currentColor" class="prefers-color-scheme-icon light d-none" viewBox="0 0 16 16">
    <path d="M8 11a3 3 0 1 1 0-6 3 3 0 0 1 0 6zm0 1a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM8 0a.5.5 0 0 1 .5.5v2a.5.5 0 0 1-1 0v-2A.5.5 0 0 1 8 0zm0 13a.5.5 0 0 1 .5.5v2a.5.5 0 0 1-1 0v-2A.5.5 0 0 1 8 13zm8-5a.5.5 0 0 1-.5.5h-2a.5.5 0 0 1 0-1h2a.5.5 0 0 1 .5.5zM3 8a.5.5 0 0 1-.5.5h-2a.5.5 0 0 1 0-1h2A.5.5 0 0 1 3 8zm10.657-5.657a.5.5 0 0 1 0 .707l-1.414 1.415a.5.5 0 1 1-.707-.708l1.414-1.414a.5.5 0 0 1 .707 0zm-9.193 9.193a.5.5 0 0 1 0 .707L3.05 13.657a.5.5 0 0 1-.707-.707l1.414-1.414a.5.5 0 0 1 .707 0zm9.193 2.121a.5.5 0 0 1-.707 0l-1.414-1.414a.5.5 0 0 1 .707-.707l1.414 1.414a.5.5 0 0 1 0 .707zM4.464 4.465a.5.5 0 0 1-.707 0L2.343 3.05a.5.5 0 1 1 .707-.707l1.414 1.414a.5.5 0 0 1 0 .708z"/>
  </svg>
  <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" fill="currentColor" class="prefers-color-scheme-icon dark d-none" viewBox="0 0 16 16">
    <path d="M6 .278a.768.768 0 0 1 .08.858 7.208 7.208 0 0 0-.878 3.46c0 4.021 3.278 7.277 7.318 7.277.527 0 1.04-.055 1.533-.16a.787.787 0 0 1 .81.316.733.733 0 0 1-.031.893A8.349 8.349 0 0 1 8.344 16C3.734 16 0 12.286 0 7.71 0 4.266 2.114 1.312 5.124.06A.752.752 0 0 1 6 .278zM4.858 1.311A7.269 7.269 0 0 0 1.025 7.71c0 4.02 3.279 7.276 7.319 7.276a7.316 7.316 0 0 0 5.205-2.162c-.337.042-.68.063-1.029.063-4.61 0-8.343-3.714-8.343-8.29 0-1.167.242-2.278.681-3.286z"/>
  </svg>
  <select class="form-select" id="prefers-color-scheme" aria-label="Select color scheme">
    <option value="system" selected>
      System
    </option>
    <option value="light">
      Light
    </option>
    <option value="dark">
      Dark
    </option>
  </select>
</div>
```

And finally the JavaScript portion that makes the selector work. This entire system checks for your computer's preferred color scheme above all else and will use that unless you manually change it.

```javascript
/**
 * dark.js
 *
 * Detects the systems current preference for dark or light mode but allows for
 * overriding the system preference.
 */

const main = document.querySelector("main");

const getSystemPreference = () => {
  if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
    return "dark";
  } else {
    return "light";
  }
};

const setIcon = (preference) => {
  const icons = document.querySelectorAll(".prefers-color-scheme-icon");
  icons.forEach((icon) => {
    icon.classList.add("d-none");
  });
  icons.forEach((icon) => {
    if (icon.classList.contains(preference)) {
      icon.classList.remove("d-none");
    }
  });
};

const setPreference = (preference) => {
  setIcon(preference);
  if (preference === "system") {
    preference = getSystemPreference();
  }
  if (preference === "dark") {
    main.classList.add("dark");
  } else {
    main.classList.remove("dark");
  }
};

const select = document.querySelector("#prefers-color-scheme");
select.addEventListener("change", () => {
  localStorage.setItem("darkMode", select.value);
  setPreference(select.value);
});

const storedPreference = localStorage.getItem("darkMode");
if (storedPreference) {
  select.value = storedPreference;
  setPreference(storedPreference);
} else {
  setPreference("system");
}
```

Once all added you'll have a selector that looks something like this that you can then swap between modes with.

![Dark mode selector](images/dark-mode-selector.webp)

You now have a fully working dark and light mode theme selector with an easy to use invert system to make future maintenance and additions easy.

Finally, **an invert word of warning**, I've found that some elements, like "position: fixed" and "position: sticky" sometimes don't perform as expected when being inverted unless you invert the "body" or "html" element of your website so always test. You may have to change how you invert based on your site's design.
