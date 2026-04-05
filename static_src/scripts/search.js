/**
 * Search.js
 *
 * Provides live search for any "id_search" field using the URL
 * /search/live/?q=<query>
 */

document.addEventListener("DOMContentLoaded", function () {
  const searchEl = document.getElementById("id_search");
  if (searchEl) {
    searchEl.addEventListener("keyup", function (e) {
      const query = searchEl.value;
      if (query.length > 0) {
        const url = "/search/live/?q=" + query;
        fetch(url)
          .then((response) => response.json())
          .then((data) => {
            // the results contain 'title', 'description', 'url'
            // add a new el to the parent of searchEl for the results
            // clear the previous el if it exists
            let resultsEl = searchEl.parentElement.querySelector("#id_search_results");
            if (resultsEl) {
              searchEl.parentElement.removeChild(resultsEl);
            }
            resultsEl = document.createElement("div");
            resultsEl.id = "id_search_results";
            searchEl.parentElement.appendChild(resultsEl);
            // add a new ul to the resultsEl
            const resultsDiv = document.createElement("ul");
            resultsDiv.classList.add("list-group", "rounded-bottom", "position-absolute", "mt-2");
            resultsDiv.style.zIndex = "1100"; // above everything else bootstrap
            resultsDiv.style.width = searchEl.offsetWidth + "px";
            resultsEl.appendChild(resultsDiv);
            // add a new li for each result
            data.forEach((result) => {
              const a = document.createElement("a");
              a.classList.add("list-group-item", "list-group-item-action");
              // strong the part of the title that matches the query
              const title = result.title.replace(
                new RegExp(query, "gi"),
                "<strong>" + query + "</strong>"
              );
              const description = result.description.replace(
                new RegExp(query, "gi"),
                "<strong class='fw-bolder'>" + query + "</strong>"
              );
              // add the title and the description, the search_descrption
              // should be in a span and muted
              a.innerHTML = "<span class='fw-bold'>" + title + "</span>" + "<br><span class='text-muted'>" + description + "</span>";
              a.href = result.url;
              resultsDiv.appendChild(a);
            });
          });
      }
    });
    // if the user clears the search field, remove the results
    searchEl.addEventListener("keyup", function (e) {
      if (searchEl.value.length === 0) {
        const resultsEl = searchEl.parentElement.querySelector("#id_search_results");
        if (resultsEl) {
          searchEl.parentElement.removeChild(resultsEl);
        }
      }
    });
    // if we are not focusing inside the search field, remove the results
    searchEl.addEventListener("blur", function (e) {
      // unless we are clicking on the results
      const searchResults = searchEl.parentElement.querySelector("#id_search_results");
      if (searchResults) {
        if (!searchResults.contains(e.relatedTarget)) {
          const resultsEl = searchEl.parentElement.querySelector("#id_search_results");
          if (resultsEl) {
            searchEl.parentElement.removeChild(resultsEl);
          }
        }
      }
    });
  }
});
