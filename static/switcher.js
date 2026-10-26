(function () {
  const MAX_RESULTS = 12;

  let posts = null;
  let overlay = null;
  let input = null;
  let list = null;
  let results = [];
  let selected = 0;

  async function loadPosts() {
    if (!posts) {
      try {
        posts = await (await fetch("/switcher.json")).json();
      } catch {
        posts = [];
      }
    }
    return posts;
  }

  function fuzzyMatch(query, text) {
    const haystack = text.toLowerCase();
    const positions = [];
    let score = 0;
    let from = 0;

    for (const char of query.toLowerCase()) {
      if (/\s/.test(char)) continue;
      const found = haystack.indexOf(char, from);
      if (found === -1) return null;
      score += positions.length && found === positions.at(-1) + 1 ? 5 : 1;
      if (found === 0 || /[\s/_-]/.test(haystack[found - 1])) score += 3;
      positions.push(found);
      from = found + 1;
    }
    return { score: score - haystack.length * 0.01, positions };
  }

  function search(query) {
    if (!query.trim()) {
      return posts.slice(0, MAX_RESULTS).map((post) => ({ post, positions: [] }));
    }

    const matches = [];
    for (const post of posts) {
      const byTitle = fuzzyMatch(query, post.title);
      const byPath = fuzzyMatch(query, post.path);
      if (!byTitle && !byPath) continue;
      const score = Math.max(byTitle?.score ?? -Infinity, (byPath?.score ?? -Infinity) - 2);
      matches.push({ post, score, positions: byTitle?.positions ?? [] });
    }
    return matches.sort((a, b) => b.score - a.score).slice(0, MAX_RESULTS);
  }

  function highlighted(text, positions) {
    const fragment = document.createDocumentFragment();
    const marked = new Set(positions);
    for (const [index, char] of [...text].entries()) {
      if (marked.has(index)) {
        const mark = document.createElement("mark");
        mark.className = "bg-transparent text-primary-color";
        mark.textContent = char;
        fragment.append(mark);
      } else {
        fragment.append(char);
      }
    }
    return fragment;
  }

  function render() {
    list.replaceChildren();
    if (results.length === 0) {
      const empty = document.createElement("li");
      empty.className = "px-4 py-3 text-zinc-500";
      empty.textContent = "No notes found";
      list.append(empty);
      input.removeAttribute("aria-activedescendant");
      return;
    }

    for (const [index, { post, positions }] of results.entries()) {
      const item = document.createElement("li");
      item.id = `quick-switcher-option-${index}`;
      item.setAttribute("role", "option");
      item.dataset.index = index;
      item.setAttribute("aria-selected", String(index === selected));
      item.className =
        "flex flex-col px-4 py-2 cursor-pointer aria-selected:bg-shade-color";

      const title = document.createElement("span");
      title.className = "truncate";
      title.append(highlighted(post.title, positions));
      const path = document.createElement("span");
      path.className = "truncate text-xs text-zinc-500";
      path.textContent = post.path;

      item.append(title, path);
      list.append(item);
    }
    input.setAttribute("aria-activedescendant", `quick-switcher-option-${selected}`);
    list.children[selected]?.scrollIntoView({ block: "nearest" });
  }

  function update() {
    results = search(input.value);
    selected = 0;
    render();
  }

  function select(index) {
    if (results.length === 0) return;
    selected = (index + results.length) % results.length;
    render();
  }

  function openPost(post) {
    close();
    htmx.ajax("GET", "/posts/" + encodeURI(post.path), {
      target: "#main-section",
      swap: "innerHTML transition:true",
      push: "true",
    });
  }

  function build() {
    overlay = document.createElement("div");
    overlay.id = "quick-switcher";
    overlay.hidden = true;
    overlay.className =
      "fixed inset-0 z-50 flex items-start justify-center px-4 pt-[15vh] bg-black/50";
    overlay.innerHTML = `
      <div role="dialog" aria-modal="true" aria-label="Quick switcher"
        class="flex flex-col w-full max-w-xl bg-background-color border border-shade-color shadow-2xl">
        <input type="text" role="combobox" aria-expanded="true" aria-autocomplete="list"
          aria-controls="quick-switcher-results" placeholder="Find a note..." autocomplete="off"
          class="w-full px-4 py-3 bg-transparent border-b border-shade-color focus:outline-hidden" />
        <ul id="quick-switcher-results" role="listbox" class="max-h-[50vh] overflow-y-auto py-1"></ul>
        <p class="px-4 py-2 text-xs text-zinc-500 border-t border-shade-color">
          ↑↓ to move · Enter to open · Esc to close
        </p>
      </div>`;
    input = overlay.querySelector("input");
    list = overlay.querySelector("ul");

    input.addEventListener("input", update);
    input.addEventListener("keydown", (event) => {
      if (event.key === "ArrowDown") select(selected + 1);
      else if (event.key === "ArrowUp") select(selected - 1);
      else if (event.key === "Enter" && results[selected]) openPost(results[selected].post);
      else if (event.key === "Escape") close();
      else return;
      event.preventDefault();
    });
    list.addEventListener("mousemove", (event) => {
      const item = event.target.closest("[data-index]");
      if (item && Number(item.dataset.index) !== selected) select(Number(item.dataset.index));
    });
    list.addEventListener("click", (event) => {
      const item = event.target.closest("[data-index]");
      if (item) openPost(results[Number(item.dataset.index)].post);
    });
    overlay.addEventListener("mousedown", (event) => {
      if (event.target === overlay) close();
    });
    document.body.append(overlay);
  }

  async function open() {
    if (!overlay) build();
    overlay.hidden = false;
    input.value = "";
    input.focus();
    await loadPosts();
    update();
  }

  function close() {
    if (overlay) overlay.hidden = true;
  }

  document.addEventListener("keydown", (event) => {
    const key = event.key.toLowerCase();
    if (!(event.ctrlKey || event.metaKey) || event.altKey || (key !== "k" && key !== "o")) return;
    event.preventDefault();
    if (overlay && !overlay.hidden) close();
    else open();
  });
})();
