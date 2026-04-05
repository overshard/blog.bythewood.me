---
title: Make your own new tab browser extension in 50 lines of code
slug: make-your-own-new-tab-browser-extension-in-50-lines-of-code
date: 2022-07-09
publish_date: 2022-07-09
tags: coding
description: There are plenty of home page and new tab replacement extensions on the Chrome Web Store that you could use, but why not make your own if it's easy?
cover_image: new-tab-cover-rev.webp
---

A benefit of making your own Chrome extension is that you also avoid all the excess tracking and bloat with a bonus of unlimited customizability. In my review of new tab extensions on the Chrome Web Store the majority of them had some form of tracking, usually in the form of Google Analytics, so I decided to just make a quick one myself!

To get started making a Chrome extension you need to create a folder and in that folder create a file called `manifest.json` with the following:

```javascript
{
  "name": "New Tab",
  "version": "1.0",
  "manifest_version": 3,
  "chrome_url_overrides": {"newtab": "newtab.html"}
}
```

In that same folder add a new file called `newtab.html`.

```html
<!DOCTYPE html>
<html>
  <head>
    <title>New Tab</title>
    <link rel="stylesheet" href="newtab.css">
  </head>
  <body>
    <div id="currentTime"></div>
    <div id="currentDate"></div>
    <script src="newtab.js"></script>
  </body>
</html>
```

From here we need two more files, one for our JavaScript and one for our CSS as we defined in our HTML above. The JavaScript file is named `newtab.js` and the CSS file is named `newtab.css`.

```javascript
const updateTime = () => {
  const currentTimeElement = document.getElementById('currentTime');
  currentTimeElement.innerHTML = new Date().toTimeString().split(' ')[0].split(':').slice(0, 2).join(':');
}
updateTime();
setInterval(updateTime, 1000);

const updateDate = () => {
  const currentDateElement = document.getElementById('currentDate');
  currentDateElement.innerHTML = new Date().toDateString().split(' ').slice(0, 3).join(' ');
}
updateDate();
setInterval(updateDate, 1000);
```

```css
body {
  background-color: #000000;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
  margin: 0;
  font-weight: lighter;
}

#currentTime {
  font-size: 20em;
  color: #ffffff;
}

#currentDate {
  font-size: 5em;
  color: #a8a8a8;
}
```

And we are good to add it to Chrome!

1. Open up the `chrome://extensions` page by navigating to the URL or clicking through the menus.
2. Enable "Developer mode" in the top right corner by clicking the switch.
3. Click "Load unpacked" under the Extensions navbar, appears after you enable "Developer mode".
4. Select the folder you put all the above code in.

If you did all that correctly you'll see a block show up that looks like this.

![New Tab extension block](images/new-tab-extension.webp)

Now if you open a new tab you'll be able to see all your work.

![New Tab extension complete](images/new-tab-extension-complete.webp)

You now have a working New Tab extension with zero trackers and you can customize it with whatever you want! In the cover photo of this post you can see my New Tab extension which you can find the [source code for on GitHub](https://github.com/overshard/newtab).

I've added a few extras to mine like:

* Bookmarks bar + bookmark edit button
* Weather icon + temperature
* Full weekday and month names
* A 12 hour clock instead of 24 hour
* Custom image background

All of which you can copy and paste out of my source code and use on your own New Tab extension if you like.

For more information on developing Chrome extensions I've found [Google's official documentation](https://developer.chrome.com/docs/extensions/) to be very helpful.

I hope to have helped you along your way to creating more custom browser extensions. It's not as hard as I originally thought and I've found I can accomplish quite a bit with just a few lines of code. I'm probably going to try making some other browser extensions in the near future.
