(function(){
  "use strict";

  fetch("device.svg")
    .then(function(r){ return r.text(); })
    .then(function(txt){
      var holder = document.createElement("div");
      holder.innerHTML = txt;
      var defs = holder.querySelector("svg");
      if (!defs) return;
      document.body.insertBefore(defs, document.body.firstChild);
      var peerTxt = txt
        .replace(/id="([\w-]+)"/g, 'id="$1-peer"')
        .replace(/url\(#([\w-]+)\)/g, "url(#$1-peer)")
        .replace(/href="#([\w-]+)"/g, 'href="#$1-peer"');
      holder.innerHTML = peerTxt;
      var peerDefs = holder.querySelector("svg");
      if (peerDefs) {
        peerDefs.classList.add("peer-defs");
        document.body.insertBefore(peerDefs, defs.nextSibling);
      }
      var uses = document.querySelectorAll("use");
      for (var i = 0; i < uses.length; i++) {
        var h = uses[i].getAttribute("href");
        uses[i].removeAttribute("href");
        uses[i].setAttribute("href", h);
      }
    })
    .catch(function(){});
})();

(function(){
  "use strict";



  var SHELLS = [
    { name:"silver",     light:"#e2e5e9", base:"#b8bcc2", shade:"#7e838b", btn:"#c2c6cc", btnHi:"#e9ecf0", etch:"rgba(0,0,0,.46)" },
    { name:"pink",       light:"#f6d3dd", base:"#e6a6b8", shade:"#a76d7e", btn:"#dbd1d5", btnHi:"#f2eaed", etch:"rgba(0,0,0,.44)" },
    { name:"light blue", light:"#dbe9f2", base:"#a8c4d8", shade:"#6d8ba1", btn:"#d0dae3", btnHi:"#edf3f8", etch:"rgba(0,0,0,.44)" },
    { name:"black",      light:"#5a5d64", base:"#34363b", shade:"#16171a", btn:"#3c3e43", btnHi:"#5c5f66", etch:"rgba(255,255,255,.42)" }
  ];

  var want = (location.search.match(/[?&]shell=([^&]+)/) || [])[1];
  var shell = SHELLS[Math.floor(Math.random() * SHELLS.length)];
  if (want) {
    want = decodeURIComponent(want).replace(/\+/g, " ").toLowerCase();
    for (var si = 0; si < SHELLS.length; si++) {
      if (SHELLS[si].name === want) { shell = SHELLS[si]; break; }
    }
  }

  var root = document.documentElement.style;
  root.setProperty("--shell-light", shell.light);
  root.setProperty("--shell-base",  shell.base);
  root.setProperty("--shell-shade", shell.shade);
  root.setProperty("--btn",         shell.btn);
  root.setProperty("--btn-hi",      shell.btnHi);
  root.setProperty("--etch",        shell.etch);

  var pose = (location.search.match(/[?&]pose=(flat|hero)/) || [])[1];
  if (pose) {
    var heroDevEl = document.querySelector(".hero-device .device");
    if (heroDevEl) heroDevEl.className = "device pose-" + pose;
  }


  var reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  var heroVid = document.querySelector(".hero-screen");
  if (heroVid) {
    if (reduced) {
      heroVid.style.opacity = "1";
    } else {
      setTimeout(function(){
        heroVid.style.opacity = "1";
        var p = heroVid.play();
        if (p && p.catch) p.catch(function(){});
      }, 1900);
      heroVid.src = "media/main.mp4";
      heroVid.load();
    }
  }


  var vid   = document.querySelector(".guide-screen");
  var gdev  = document.querySelector(".guide-stage .device");
  var steps = Array.prototype.slice.call(document.querySelectorAll(".step"));

  var stage = gdev ? gdev.parentNode : null;
  var peer = null, pvid = null;
  if (gdev) {
    peer = gdev.cloneNode(true);
    peer.classList.add("is-peer");
    peer.classList.remove("is-waiting", "is-shut", "is-swinging");
    peer.setAttribute("aria-hidden", "true");
    var others = SHELLS.filter(function(x){ return x !== shell; });
    var their = others[Math.floor(Math.random() * others.length)];
    var peerVars = { "light":their.light, "base":their.base, "shade":their.shade,
                     "btn":their.btn, "btn-hi":their.btnHi };
    for (var pk in peerVars) root.setProperty("--peer-" + pk, peerVars[pk]);
    peer.style.setProperty("--shell-light", their.light);
    peer.style.setProperty("--shell-base",  their.base);
    peer.style.setProperty("--shell-shade", their.shade);
    peer.style.setProperty("--btn",         their.btn);
    peer.style.setProperty("--btn-hi",      their.btnHi);
    peer.style.setProperty("--etch",        their.etch);
    var peerUses = peer.querySelectorAll("use");
    for (var pu = 0; pu < peerUses.length; pu++) {
      peerUses[pu].setAttribute("href", peerUses[pu].getAttribute("href") + "-peer");
    }
    pvid = peer.querySelector("video");
    pvid.classList.remove("guide-screen");
    stage.appendChild(peer);
  }

  var pairOn = false, pairStarting = false;
  function play(v){ var p = v.play(); if (p && p.catch) p.catch(function(){}); }
  function pairReady(){ return vid.readyState >= 3 && pvid.readyState >= 3; }
  function startPair(){
    if (!pairOn || pairStarting || !pairReady()) return;
    pairStarting = true;
    vid.pause(); pvid.pause();
    pvid.playbackRate = 1;
    var left = 2, done = false;
    function go(){
      if (done || --left > 0) return;
      done = true;
      pairStarting = false;
      if (!pairOn) return;
      play(vid); play(pvid);
    }
    vid.addEventListener("seeked", go, { once:true });
    pvid.addEventListener("seeked", go, { once:true });
    vid.currentTime = 0; pvid.currentTime = 0;
    setTimeout(function(){ left = 1; go(); }, 600);
  }
  if (vid && pvid) {
    ["canplay", "canplaythrough"].forEach(function(e){
      [vid, pvid].forEach(function(v){
        v.addEventListener(e, function(){ if (vid.paused || pvid.paused) startPair(); });
      });
    });
    vid.addEventListener("waiting", function(){ if (pairOn) pvid.pause(); });
    pvid.addEventListener("waiting", function(){ if (pairOn) vid.pause(); });
    vid.addEventListener("ended", function(){ if (pairOn) startPair(); });
    vid.addEventListener("timeupdate", function(){
      if (!pairOn || pairStarting || vid.paused) return;
      if (pvid.paused) { play(pvid); return; }
      var d = vid.currentTime - pvid.currentTime;
      if (Math.abs(d) > 0.3) { startPair(); return; }
      pvid.playbackRate = 1 + Math.max(-0.1, Math.min(0.1, d * 0.5));
    });
  }
  function showPeer(clip){
    if (!stage || !pvid) return;
    stage.classList.toggle("is-pair", !!clip);
    pairOn = !!clip && !reduced;
    vid.loop = !pairOn;
    if (!clip) { pvid.pause(); pvid.removeAttribute("src"); pvid.playbackRate = 1; return; }
    pvid.poster = "media/" + clip.replace(/\.mp4$/, ".webp");
    if (reduced) { pvid.removeAttribute("src"); return; }
    if (pvid.getAttribute("src") !== "media/" + clip) {
      pvid.src = "media/" + clip;
      pvid.load();
    }
  }

  var current = null;

  var LID_SWING_MS = 1150;
  var swingTimer = null;

  var playRetries = [];
  function ensurePlaying(){
    if (reduced || !vid || !vid.getAttribute("src") || !vid.paused) return;
    if (pairOn) { startPair(); return; }
    var p = vid.play();
    if (p && p.catch) p.catch(function(){});
  }
  function nudgePlayback(){
    for (var i = 0; i < playRetries.length; i++) clearTimeout(playRetries[i]);
    playRetries = [];
    ensurePlaying();
    playRetries.push(setTimeout(ensurePlaying, 220), setTimeout(ensurePlaying, 800));
  }
  if (vid) {
    vid.loop = !pairOn;
    vid.addEventListener("loadeddata", ensurePlaying);
    vid.addEventListener("canplay", ensurePlaying);
    vid.addEventListener("canplaythrough", ensurePlaying);
  }

  function show(step){
    if (!vid || !step || step === current) return;
    current = step;
    var at = 0;
    for (var i = 0; i < steps.length; i++) {
      var on = steps[i] === step;
      steps[i].classList.toggle("is-active", on);
      if (on) at = i;
    }
    if (stepAt) stepAt.textContent = String(at + 1);
    if (prevBtn) prevBtn.disabled = at === 0;
    if (nextBtn) nextBtn.disabled = at === steps.length - 1;
    if (gdev) {
      var wantShut = step.getAttribute("data-lid") === "shut";
      if (!wantShut && gdev.classList.contains("is-shut") && !reduced) {
        gdev.classList.add("is-swinging");
        clearTimeout(swingTimer);
        swingTimer = setTimeout(function(){ gdev.classList.remove("is-swinging"); }, LID_SWING_MS);
      }
      gdev.classList.toggle("is-shut", wantShut);
    }

    showPeer(step.getAttribute("data-pair"));
    var only = step.getAttribute("data-still");
    if (only) {
      vid.removeAttribute("src");
      vid.load();
      vid.poster = "media/" + only;
      vid.style.opacity = "1";
      return;
    }
    var clip = step.getAttribute("data-clip");
    if (!clip) { vid.style.opacity = "0"; vid.removeAttribute("src"); return; }
    var still = "media/" + clip.replace(/\.mp4$/, ".webp");
    if (reduced) {
      if (vid.poster !== still) { vid.removeAttribute("src"); vid.poster = still; }
      vid.style.opacity = "1";
      return;
    }
    var path = "media/" + clip;
    if (vid.getAttribute("src") === path) {
      vid.style.opacity = "1";
      nudgePlayback();
      return;
    }
    vid.poster = still;
    vid.src = path;
    vid.style.opacity = "1";
    vid.load();
    nudgePlayback();
  }

  var stepList = document.querySelector(".guide-steps");
  var prevBtn  = document.getElementById("step-prev");
  var nextBtn  = document.getElementById("step-next");
  var stepAt   = document.getElementById("step-at");
  var stepOf   = document.getElementById("step-of");
  if (stepList && steps.length) stepList.classList.add("is-live");
  if (stepOf) stepOf.textContent = String(steps.length);

  function stepBy(delta){
    var at = steps.indexOf(current);
    var to = at + delta;
    if (at < 0 || to < 0 || to >= steps.length) return;
    show(steps[to]);
    setHash(steps[to].id);
  }
  if (prevBtn) prevBtn.addEventListener("click", function(){ stepBy(-1); });
  if (nextBtn) nextBtn.addEventListener("click", function(){ stepBy(1); });

  var guidePanel = document.getElementById("features");
  if (guidePanel) guidePanel.addEventListener("keydown", function(e){
    if (e.target.closest("[role=tablist]")) return;
    if (e.key === "ArrowLeft") { e.preventDefault(); stepBy(-1); }
    else if (e.key === "ArrowRight") { e.preventDefault(); stepBy(1); }
  });


  function wire(list){
    var tabs = Array.prototype.slice.call(list.querySelectorAll("[role=tab]"));
    var group = {
      tabs: tabs,
      panels: tabs.map(function(t){ return document.getElementById(t.getAttribute("aria-controls")); }),
      select: function(tab, focus){
        var i = tabs.indexOf(tab);
        if (i < 0) return;
        for (var k = 0; k < tabs.length; k++) {
          var on = k === i;
          tabs[k].setAttribute("aria-selected", on ? "true" : "false");
          tabs[k].tabIndex = on ? 0 : -1;
          if (group.panels[k]) group.panels[k].hidden = !on;
        }
        if (hero) hero.hidden = true;
        if (tabs[i].scrollIntoView && list.scrollWidth > list.clientWidth + 1) {
          try { tabs[i].scrollIntoView({ inline: "center", block: "nearest" }); } catch (e) {}
        }
        if (focus) tabs[i].focus();
      }
    };
    for (var i = 0; i < tabs.length; i++) {
      (function(tab){
        tab.addEventListener("click", function(){
          group.select(tab, false);
          setHash(tab.getAttribute("aria-controls"));
        });
      })(tabs[i]);
    }
    list.addEventListener("keydown", function(e){
      var at = tabs.indexOf(document.activeElement);
      if (at < 0) return;
      var to = -1;
      if (e.key === "ArrowRight" || e.key === "ArrowDown") to = (at + 1) % tabs.length;
      else if (e.key === "ArrowLeft" || e.key === "ArrowUp") to = (at - 1 + tabs.length) % tabs.length;
      else if (e.key === "Home") to = 0;
      else if (e.key === "End") to = tabs.length - 1;
      if (to < 0) return;
      e.preventDefault();
      group.select(tabs[to], true);
      setHash(tabs[to].getAttribute("aria-controls"));
    });
    return group;
  }

  var hero = document.getElementById("hero");
  var groups = Array.prototype.slice.call(document.querySelectorAll("[role=tablist]")).map(wire);


  var menuBtn = document.getElementById("menu-btn");
  var tabsNav = document.getElementById("section-tabs");
  function setMenu(open){
    if (!menuBtn || !tabsNav) return;
    tabsNav.classList.toggle("is-open", open);
    menuBtn.setAttribute("aria-expanded", open ? "true" : "false");
  }
  if (menuBtn && tabsNav) {
    menuBtn.addEventListener("click", function(e){
      e.stopPropagation();
      setMenu(!tabsNav.classList.contains("is-open"));
    });
    tabsNav.addEventListener("click", function(){ setMenu(false); });
    document.addEventListener("click", function(e){
      if (e.target !== menuBtn && !menuBtn.contains(e.target) && !tabsNav.contains(e.target)) {
        setMenu(false);
      }
    });
    document.addEventListener("keydown", function(e){
      if (e.key !== "Escape" || !tabsNav.classList.contains("is-open")) return;
      setMenu(false);
      menuBtn.focus();
    });
  }

  function showHero(){
    var grp = groups[0];
    if (!grp) return;
    for (var k = 0; k < grp.tabs.length; k++) {
      grp.tabs[k].setAttribute("aria-selected", "false");
      grp.tabs[k].tabIndex = k === 0 ? 0 : -1;
    }
    for (var q = 0; q < grp.panels.length; q++) {
      if (grp.panels[q]) grp.panels[q].hidden = true;
    }
    if (hero) hero.hidden = false;
  }

  var mark = document.querySelector(".mast-mark");
  if (mark) mark.addEventListener("click", function(e){
    e.preventDefault();
    showHero();
    setHash("hero");
  });


  var routing = false;
  function setHash(id){
    if (!id) return;
    routing = true;
    if (history.replaceState) history.replaceState(null, "", "#" + id);
    else location.hash = id;
    routing = false;
  }

  function reveal(id){
    var el = id && document.getElementById(id);
    if (!el) return false;
    if (el === hero) { showHero(); return true; }
    var chain = [];
    for (var node = el; node && node !== document.body; node = node.parentNode) {
      for (var g = 0; g < groups.length; g++) {
        var at = groups[g].panels.indexOf(node);
        if (at >= 0) chain.unshift({ group: groups[g], tab: groups[g].tabs[at] });
      }
    }
    for (var c = 0; c < chain.length; c++) chain[c].group.select(chain[c].tab, false);
    if (el.classList.contains("step")) show(el);
    return true;
  }

  window.addEventListener("hashchange", function(){
    if (routing) return;
    reveal(location.hash.slice(1));
  });

  if (steps.length) show(steps[0]);
  reveal(location.hash.slice(1));

  var say = document.querySelector(".say");
  var voice = document.getElementById("say-slot");
  if (say && voice) {
    say.addEventListener("click", function(){
      voice.currentTime = 0;
      var playing = voice.play();
      if (playing && playing.catch) playing.catch(function(){});
    });
    voice.addEventListener("play", function(){ say.classList.add("is-playing"); });
    ["pause", "ended"].forEach(function(e){
      voice.addEventListener(e, function(){ say.classList.remove("is-playing"); });
    });
  }

})();
