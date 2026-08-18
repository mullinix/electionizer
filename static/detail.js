(function () {
  var root = document.getElementById("detail-stream");
  if (!root) return;

  var url = root.getAttribute("data-stream-url");
  if (!url) return;

  var bar = document.getElementById("detail-bar");
  var pctEl = document.getElementById("detail-pct");
  var msgEl = document.getElementById("detail-msg");
  var stagesEl = document.getElementById("detail-stages");
  var shell = document.getElementById("detail-shell");
  var bodyRoot = document.getElementById("detail-root");
  var progress = document.getElementById("detail-progress");
  var es = null;
  var total = 0;
  var closed = false;

  function setPct(completed, tot, label) {
    total = tot || total;
    var p = total ? Math.min(100, Math.round((completed * 100) / total)) : 0;
    if (bar) bar.style.width = p + "%";
    if (pctEl) {
      pctEl.innerHTML =
        "<strong>" +
        completed +
        " / " +
        total +
        "</strong> · " +
        (label || "working…");
    }
  }

  function stageRow(id) {
    if (!stagesEl) return null;
    return stagesEl.querySelector('[data-stage-id="' + id + '"]');
  }

  function markStage(id, status, detail) {
    var li = stageRow(id);
    if (!li) return;
    li.className = "stage-" + status;
    var mark = li.querySelector(".stage-mark");
    if (mark) {
      if (status === "running") mark.textContent = "›";
      else if (status === "done") mark.textContent = "✓";
      else if (status === "skip") mark.textContent = "–";
      else if (status === "error") mark.textContent = "!";
      else mark.textContent = "·";
    }
    var d = li.querySelector(".stage-detail");
    if (d) d.textContent = detail ? " · " + detail : "";
  }

  function fail(message) {
    if (msgEl) msgEl.textContent = message || "Failed to load detail.";
    if (progress) progress.classList.add("status-failed");
    closeEs();
  }

  function finishHtml(html) {
    if (bodyRoot) {
      bodyRoot.innerHTML = html;
      bodyRoot.hidden = false;
    }
    if (shell) shell.hidden = true;
    if (progress) progress.hidden = true;
    closeEs();
  }

  function closeEs() {
    closed = true;
    if (es) {
      try {
        es.close();
      } catch (_) {}
      es = null;
    }
  }

  function onPlan(data) {
    total = data.total || 0;
    setPct(0, total, "starting…");
    if (msgEl) msgEl.textContent = "Running " + total + " data step(s)…";
  }

  function onStage(data) {
    markStage(data.id, data.status, data.detail);
    if (data.status === "running") {
      setPct(data.completed || 0, data.total, data.label);
      if (msgEl) msgEl.textContent = data.label + "…";
    } else {
      setPct(data.completed || 0, data.total, data.label);
      if (msgEl) {
        msgEl.textContent =
          data.label +
          (data.detail ? " — " + data.detail : "") +
          (data.completed >= data.total ? "" : "…");
      }
    }
  }

  try {
    es = new EventSource(url);
  } catch (e) {
    fail("Could not open detail stream.");
    return;
  }

  es.addEventListener("plan", function (ev) {
    try {
      onPlan(JSON.parse(ev.data));
    } catch (_) {}
  });

  es.addEventListener("stage", function (ev) {
    try {
      onStage(JSON.parse(ev.data));
    } catch (_) {}
  });

  es.addEventListener("html", function (ev) {
    try {
      var data = JSON.parse(ev.data);
      if (data && data.html) finishHtml(data.html);
    } catch (_) {
      fail("Bad detail payload.");
    }
  });

  es.addEventListener("done", function () {
    closeEs();
    if (shell && !shell.hidden && bodyRoot && !bodyRoot.innerHTML) {
      fail("Stream ended without content.");
    }
  });

  es.addEventListener("fail", function (ev) {
    var msg = "Load failed.";
    try {
      var d = JSON.parse(ev.data);
      if (d.message) msg = d.message;
    } catch (_) {}
    fail(msg);
  });

  es.onerror = function () {
    if (closed) return;
    // Browser may retry; only fail if we never got HTML
    if (bodyRoot && bodyRoot.innerHTML) {
      closeEs();
      return;
    }
  };

  window.addEventListener("pagehide", closeEs);
  window.addEventListener("beforeunload", closeEs);
})();
