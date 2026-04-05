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
