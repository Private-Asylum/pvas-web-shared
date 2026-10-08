/*! yeti-css 7.0.0-alpha.0 | FSL-1.1-MIT | https://foundationcss.com/yeti/ */
// Yeti 7.0.0-alpha.0: every optional module in one file. Load with <script type="module">.

// alert.js
{
// Alert: a click on the close button fades the alert out and removes it.
// Delegated, so alerts added after load work too; safe on pages with none.
// The duration is the fast token, which reduced motion collapses.
document.addEventListener('click', (event) => {
	// The page's own listener ran first and asked for nothing to happen.
	if (event.defaultPrevented) return;
	const button = event.target?.closest?.('.alert > [data-close]');
	if (!button) return;
	const alert = button.parentElement;
	const parent = alert.parentElement;
	const held = alert.contains(document.activeElement);
	const duration = parseFloat(getComputedStyle(alert).getPropertyValue('--yeti-duration-fast')) || 0;
	alert.animate([{ opacity: 1 }, { opacity: 0 }], { duration, fill: 'forwards' }).finished.then(() => {
		// Announced before the alert leaves the page, so a listener can still
		// read the element it is about: bubbles, so one listener on document
		// hears every alert, and composed, so it crosses out of a shadow root.
		alert.dispatchEvent(new CustomEvent('yeti:close', { bubbles: true, composed: true }));
		alert.remove();
		// The button that had focus has just gone, so put focus where the alert
		// was rather than letting it fall to the top of the document.
		// Only add tabindex if the parent isn't already focusable, and only
		// remove it again if we added it, so an author's own tabindex (a focus
		// trap, a scrollable region) survives untouched.
		if (held && parent) {
			const had = parent.getAttribute('tabindex');
			if (had === null) parent.setAttribute('tabindex', '-1');
			parent.focus({ preventScroll: true });
			if (had === null) parent.addEventListener('blur', () => parent.removeAttribute('tabindex'), { once: true });
		}
	});
});
}

// carousel.js
{
// Carousel dots. The dots are links to slide ids so that the component works
// with no script at all, but following a link to a fragment is a navigation,
// and every navigation adds an entry to the browser's history. A reader who
// looked at four slides then pressed back four times to leave the page is not
// going to forgive that, and nothing in CSS opts out of it.
//
// So this module takes the click and scrolls the track itself. Without it the
// dots still work, exactly as before; the difference is only the history.
//
// Delegated, so dots added after load work too, and safe on pages with none.
document.addEventListener('click', (event) => {
	const dot = event.target?.closest?.('.carousel > [data-dots] a[href^="#"]');
	if (!dot) return;
	// Modified clicks belong to the browser: a new tab or window is a real
	// navigation and the reader asked for it.
	if (event.defaultPrevented || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;

	const carousel = dot.closest('.carousel');
	const track = carousel?.querySelector(':scope > [data-track]');
	const slide = document.getElementById(decodeURIComponent(dot.getAttribute('href').slice(1)));
	// Only a dot pointing at a slide of its own carousel is ours to handle.
	// Anything else is left to the browser rather than silently swallowed.
	if (!track || !slide || !track.contains(slide)) return;

	event.preventDefault();
	// Measured from the boxes rather than offsetLeft, which depends on whichever
	// ancestor happens to be positioned, and against the start edge for the
	// writing direction, since that is the edge the slides snap to. The target
	// is absolute: a relative scrollBy is resolved against a smooth scroll still
	// in flight, and WebKit then overshoots and does not re-snap, so two dots
	// pressed in quick succession parked the track between slides. No behavior
	// is passed, so the scroll takes the track's own scroll-behavior, and that
	// reads the token that reduced motion collapses.
	const rtl = getComputedStyle(track).direction === 'rtl';
	const [own, target] = [track.getBoundingClientRect(), slide.getBoundingClientRect()];
	track.scrollTo({ left: track.scrollLeft + (rtl ? target.right - own.right : target.left - own.left) });

	// The index is the slide's place among its own track's slides, which is
	// what a counter or a caption beside the carousel needs; the scroll above
	// may still be travelling, and the event is about the choice, not the
	// arrival.
	const slides = [...track.querySelectorAll(':scope > [data-slide]')];
	carousel.dispatchEvent(new CustomEvent('yeti:slide', {
		bubbles: true,
		composed: true,
		detail: { index: slides.indexOf(slide), slide },
	}));
});
}

// demo.js
{
// Demo frames. A framed demo carries its example twice, once escaped into the
// iframe's srcdoc and once as code under the box, and an author writing one
// by hand has to keep the two in step. The docs generator does that for the
// site; this module does it for everyone else: the author writes the code
// once, in the pre, and the frame is built from it. Without the module the
// code still shows and the box stays empty, so nothing depends on it.
//
// The framed document is a whole Yeti page: the host page's own Yeti
// stylesheet, found by its file name, unless the figure names another with
// data-stylesheet, and yeti.js from the folder beside it, so an example that
// needs a module has it. A host page in someone else's colors points at a
// plain yeti.css that way; host yeti.js next to it.
//
// It also gives each box a grip on its end edge, a separator a mouse, a
// finger or the keyboard can move, because the browser's own resize corner
// is invisible in Safari and does nothing for touch or keys. Without the
// module the box keeps that native corner.
// Safe on pages with no demo, and demos added later are filled as they land.

// The resize logic, kept apart from the page so a Node test can call it.
// Widths are CSS pixels of the box's content box, the size the width tokens
// and the stop label measure.

// A drag moves the box's end edge with the pointer: rightward in a
// left-to-right page, leftward in a right-to-left one, never past the box's
// own min and max.
function widthFromDrag(startPx, dx, rtl, minPx, maxPx) {
	return Math.min(maxPx, Math.max(minPx, startPx + (rtl ? -dx : dx)));
}

// A key steps to the next stop strictly beyond the current width, up
// (direction 1) or down (-1); past the last stop that fits, the box goes to
// its max or min. A width a hair off a stop counts as on it.
function stepWidth(currentPx, direction, stopsPx, minPx, maxPx) {
	const beyond = direction > 0
		? stopsPx.filter((stop) => stop > currentPx + 0.5)
		: stopsPx.filter((stop) => stop < currentPx - 0.5);
	const next = beyond.length ? (direction > 0 ? Math.min(...beyond) : Math.max(...beyond)) : (direction > 0 ? maxPx : minPx);
	return Math.min(maxPx, Math.max(minPx, next));
}

// The stop a width is at, named exactly as the bar's label names it: the
// largest stop at or below the width, and xs for anything below sm.
function stopName(px, stops) {
	const floor = stops.find((stop) => stop.name === 'sm')?.px ?? 0;
	let name = 'xs';
	for (const stop of stops) if (stop.px >= floor && px >= stop.px) name = stop.name;
	return name;
}

const stylesheetFor = (figure) => figure.dataset.stylesheet
	|| document.querySelector('link[rel="stylesheet"][href$="yeti.css"]')?.href
	|| '';

function fill(figure) {
	const box = figure.querySelector(':scope > [data-preview]');
	const code = figure.querySelector(':scope > details > pre > code, :scope > details > pre');
	if (!box || !code) return;
	let frame = box.querySelector(':scope > iframe');
	// An empty box asks for its frame to be made; the marker's value names it.
	if (!frame && box.childElementCount === 0) {
		frame = document.createElement('iframe');
		frame.title = `${box.dataset.preview || 'Example'}, live`;
		box.append(frame);
	}
	// A frame the author already filled, or a box holding direct markup, is left alone.
	if (!frame || frame.hasAttribute('srcdoc')) return;
	const href = stylesheetFor(figure);
	if (!href) return;
	// Relative paths in the example resolve beside the stylesheet, as the
	// generator arranges, so a picture the docs host is found from any page.
	// The stylesheet may itself be a relative path, so it is resolved first.
	const base = new URL('.', new URL(href, document.baseURI)).href;
	frame.srcdoc = `<base href="${base}"><link rel="stylesheet" href="${href}"><script type="module" src="${base}yeti.js"></script><body style="margin:0;padding:var(--yeti-space-md)">${code.textContent.trim()}`;
}

// The width stops, in rem: the width tokens' defaults, the same thresholds
// the bar's label flips at.
const stopsRem = [['2xs', 12], ['xs', 16], ['sm', 24], ['md', 32], ['lg', 48], ['xl', 64], ['2xl', 80]];
const stops = () => {
	const rem = parseFloat(getComputedStyle(document.documentElement).fontSize) || 16;
	return stopsRem.map(([name, size]) => ({ name, px: size * rem }));
};
// The box is content-box, so its computed inline size is the content width.
const contentWidth = (box) => parseFloat(getComputedStyle(box).inlineSize);
// The box's own min and max, read by asking for nothing and for everything,
// so they are whatever the stylesheet says, caps and container included.
function limits(box) {
	const authored = box.style.inlineSize;
	box.style.inlineSize = '0px';
	const min = contentWidth(box);
	box.style.inlineSize = '100000px';
	const max = contentWidth(box);
	box.style.inlineSize = authored;
	return { min, max };
}

function grip(figure) {
	const box = figure.querySelector(':scope > [data-preview]');
	if (!box || figure.querySelector(':scope > [role="separator"]')) return;
	const handle = document.createElement('div');
	handle.setAttribute('role', 'separator');
	handle.setAttribute('aria-orientation', 'vertical');
	handle.setAttribute('aria-label', `Resize ${box.dataset.preview || 'example'}`);
	handle.tabIndex = 0;
	box.after(handle);
	let bounds = limits(box);
	const rtl = () => getComputedStyle(box).direction === 'rtl';
	// The values in pixels, and the box's outer edge for the stylesheet to
	// put the grip on, kept current however the box's size changes.
	const update = () => {
		const now = contentWidth(box);
		handle.setAttribute('aria-valuemin', Math.round(bounds.min));
		handle.setAttribute('aria-valuemax', Math.round(bounds.max));
		handle.setAttribute('aria-valuenow', Math.round(now));
		handle.setAttribute('aria-valuetext', `${stopName(now, stops())}, ${Math.round(now)} pixels`);
		// The box's end edge from the figure's inline start, and its middle
		// from the figure's top; the figure is the grip's containing block.
		const edge = rtl() ? figure.clientWidth - box.offsetLeft : box.offsetLeft + box.offsetWidth;
		handle.style.setProperty('--_yeti-demo-edge', `${edge}px`);
		handle.style.setProperty('--_yeti-demo-middle', `${box.offsetTop + box.offsetHeight / 2}px`);
	};
	const set = (px) => {
		box.style.inlineSize = `${px}px`;
		update();
	};
	new ResizeObserver(update).observe(box);
	// The container changing width moves the max.
	new ResizeObserver(() => { bounds = limits(box); update(); }).observe(figure);

	handle.addEventListener('pointerdown', (event) => {
		if (event.button !== 0) return;
		event.preventDefault();
		handle.focus();
		bounds = limits(box);
		const start = contentWidth(box);
		const from = event.clientX;
		const backwards = rtl();
		// The grip captures the pointer, so a frame under it cannot take the
		// drag. The frame's pointer-events are left alone: turning them off for
		// the drag left Chromium sending the wheel to the page, not the frame,
		// for a while after.
		const move = (e) => set(widthFromDrag(start, e.clientX - from, backwards, bounds.min, bounds.max));
		handle.addEventListener('pointermove', move);
		handle.addEventListener('lostpointercapture', () => {
			handle.removeEventListener('pointermove', move);
		}, { once: true });
		handle.setPointerCapture(event.pointerId);
	});

	handle.addEventListener('keydown', (event) => {
		const { key } = event;
		if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(key)) return;
		event.preventDefault();
		bounds = limits(box);
		if (key === 'Home') return set(bounds.min);
		if (key === 'End') return set(bounds.max);
		const direction = (key === 'ArrowRight' ? 1 : -1) * (rtl() ? -1 : 1);
		set(stepWidth(contentWidth(box), direction, stops().map((stop) => stop.px), bounds.min, bounds.max));
	});
}

const enhance = (figure) => {
	fill(figure);
	grip(figure);
};
const enhanceAll = (root) => root.querySelectorAll?.('.demo').forEach(enhance);
enhanceAll(document);
new MutationObserver((records) => {
	for (const record of records) for (const node of record.addedNodes) {
		if (node.nodeType !== Node.ELEMENT_NODE) continue;
		if (node.matches('.demo')) enhance(node);
		enhanceAll(node);
	}
}).observe(document.body, { childList: true, subtree: true });
}

// dialog.js
{
// Dialog: the browser opens it. A button carrying commandfor="<id>" and
// command="show-modal" calls showModal on that dialog itself, which is what
// makes the page behind it inert, holds focus inside, and closes on Escape.
// This module adds the two things the platform does not do yet. A click on
// the backdrop closes the dialog: closedby="any" would, but Safari lacks it
// and it is not Baseline. Focus returns to the button that opened it: the
// browser restores focus to whatever had it when the dialog opened, and in
// WebKit a clicked button never has it. Delegated, so dialogs added after
// load work too, and safe on a page with none.
// The opener is remembered per dialog rather than once for the page, so a
// dialog opened from inside another still returns focus to its own trigger.
const openers = new WeakMap();
// Whether the last press on each dialog began outside its box. A click whose
// press and release land on different elements is delivered to their common
// ancestor with the release's coordinates, so a drag that starts on text in
// the dialog and ends over the backdrop looks exactly like a backdrop click.
// Closing needs both ends outside.
const pressedOutside = new WeakMap();
const outside = (dialog, event) => {
	const box = dialog.getBoundingClientRect();
	return event.clientX < box.left || event.clientX > box.right || event.clientY < box.top || event.clientY > box.bottom;
};

// A command event is dispatched on the dialog and does not bubble, so the
// document hears it only on the capture phase. An already open dialog is left
// alone: the browser ignores the command, and a second listener would only
// overwrite the opener with the wrong button.
document.addEventListener('command', (event) => {
	const dialog = event.target?.closest?.('dialog.dialog');
	if (!dialog || dialog.open || event.command !== 'show-modal' || !event.source) return;
	// The opener is recorded here, before the default action, because that is
	// what moves focus off the button; by the time the dialog is open the
	// button that was pressed is no longer knowable.
	openers.set(dialog, event.source);
	// The command event fires before the browser acts on it, and a listener may
	// still cancel it, so nothing is claimed here. The default action runs later
	// in the same task, which makes a task the earliest point at which the
	// dialog is known to be open. A microtask is too early for a trusted event:
	// its checkpoint is reached with the stack already empty, before the default
	// action, and the guard below would find the dialog still shut and say
	// nothing. Registering the close listener here too means a cancelled command
	// leaks nothing, since the listener only exists once the dialog really opened.
	setTimeout(() => {
		if (!dialog.open) return;
		dialog.addEventListener('close', () => {
			openers.get(dialog)?.focus?.({ preventScroll: true });
			openers.delete(dialog);
		}, { once: true });
		dialog.dispatchEvent(new CustomEvent('yeti:open', { bubbles: true, composed: true }));
	});
}, { capture: true });

document.addEventListener('pointerdown', (event) => {
	const dialog = event.target?.closest?.('dialog.dialog');
	if (dialog?.open) pressedOutside.set(dialog, outside(dialog, event));
}, { capture: true });

document.addEventListener('click', (event) => {
	// A click on the backdrop reports the dialog as its target but lands
	// outside the dialog's own box. A keyboard activation has no coordinates
	// at all, so it must not be mistaken for one.
	const dialog = event.target?.closest?.('dialog.dialog');
	if (!dialog || !dialog.open || event.detail === 0) return;
	if (outside(dialog, event) && pressedOutside.get(dialog)) dialog.close();
});

// close does not bubble, so the document hears it only on the capture phase.
// Every close passes through here, whether it came from Escape, a form
// button, the backdrop click above, or a script calling close() — so the
// event does not depend on how the dialog was opened, the way the focus
// return above does.
document.addEventListener('close', (event) => {
	const dialog = event.target?.closest?.('dialog.dialog');
	if (dialog) dialog.dispatchEvent(new CustomEvent('yeti:close', { bubbles: true, composed: true }));
}, { capture: true });
}

// enter.js
{
// Enter: data-once. enter.css gives a `.enter[data-once]` element (and, with
// data-stagger, each of its children) `animation: none`, so it simply sits
// there, visible, until this module says otherwise; without it that is where
// it stays, present but never animated, rather than stranded invisible.
//
// One IntersectionObserver for every .enter[data-once] already in the
// document: the first time an entry is near the viewport, removing its
// data-once attribute is the whole trick. That drops the `animation: none`
// rule out of the match, and the ordinary .enter rules underneath it apply
// exactly as if data-once had never been there. A CSS animation only plays
// when the style that names it changes, and this is that one change: it
// happens once, because the attribute is gone and nothing here ever puts it
// back, so scrolling away and back has nothing left to replay.
//
// rootMargin grows the root 10% of the viewport past its real bottom edge,
// so intersection starts a little before the element is actually on screen:
// the arrival is already under way by the time the reader's eye reaches it,
// rather than starting only once it has fully crossed the fold.
//
// Elements added after load are not picked up, as with toc.js and tabs.js:
// the observer is wired once, over what is on the page when this module
// runs.
const once = new IntersectionObserver((entries) => {
	for (const entry of entries) {
		if (!entry.isIntersecting) continue;
		entry.target.removeAttribute('data-once');
		once.unobserve(entry.target);
	}
}, { rootMargin: '0px 0px 10% 0px' });

for (const el of document.querySelectorAll('.enter[data-once]')) once.observe(el);
}

// hover.js
{
// Dropdown, opening on hover. Nothing but script can open a popover, so a menu
// that wants to open under the pointer needs this. It is opt in per instance
// through data-trigger="hover"; every other dropdown keeps its click, and the
// click path here is untouched in either case. Delegated on pointerover and
// pointerout, which bubble where pointerenter and pointerleave do not, so
// dropdowns added after load work too.
//
// The module keys off the attribute rather than off .dropdown, so any later
// component built on a popover gets this for free by declaring data-trigger.
//
// It is also a stopgap with a stated end. The interestfor attribute is exactly
// this feature, standardised, and it is in one engine today. When it reaches
// Baseline this file is deleted and data-trigger="hover" maps to it instead.
const fine = matchMedia('(hover: hover) and (pointer: fine)');
const timers = new WeakMap();
// Only a panel this module opened is a panel this module may close. Otherwise a
// stray sweep of the pointer shuts a menu someone opened with Enter, while
// their focus is still sitting on the trigger.
const opened = new WeakSet();
// Panels that already carry the listener below, so it is attached once each.
const wired = new WeakSet();

const partsOf = (wrapper) => {
	// The declared relationship is the source of truth, as tabs.js does with
	// aria-controls: find the panel the trigger names, never just any popover.
	const trigger = wrapper.querySelector(':scope > [popovertarget]');
	const panel = trigger && document.getElementById(trigger.getAttribute('popovertarget'));
	return panel?.showPopover ? { trigger, panel } : null;
};

// A theme tunes the feel without touching script, the way alert.js reads its
// duration. An absent token means no wait, which is the sane default.
const delay = (el, name) => parseFloat(getComputedStyle(el).getPropertyValue(name)) || 0;

const schedule = (wrapper, ms, run) => {
	clearTimeout(timers.get(wrapper));
	timers.set(wrapper, setTimeout(run, ms));
};

// A move that stays inside the wrapper is not a crossing. The panel is a DOM
// child of the wrapper even while it paints in the top layer, so hovering the
// panel itself keeps the menu open with no special handling.
const crossing = (event) => {
	const wrapper = event.target?.closest?.('[data-trigger="hover"]');
	if (!wrapper || wrapper.contains(event.relatedTarget)) return null;
	return wrapper;
};

document.addEventListener('pointerover', (event) => {
	// Read at event time, not at load: a tablet that gains a mouse is handled.
	if (!fine.matches) return;
	const wrapper = crossing(event);
	const parts = wrapper && partsOf(wrapper);
	if (!parts) return;
	schedule(wrapper, delay(wrapper, '--yeti-dropdown-open-delay'), () => {
		// showPopover throws when the popover is already open.
		if (parts.panel.matches(':popover-open')) return;
		parts.panel.showPopover();
		opened.add(parts.panel);
		// However it closes next, from here, Escape, or a click outside, it
		// stops being ours, so a later click-open is not ours to undo. The
		// listener is persistent, not once: showPopover queues its own open
		// toggle, which fires after this line runs and would consume a
		// once-listener before the close it was waiting for ever arrived.
		if (!wired.has(parts.panel)) {
			wired.add(parts.panel);
			parts.panel.addEventListener('toggle', (change) => {
				if (change.newState === 'closed') opened.delete(parts.panel);
			});
		}
	});
});

// A press cancels any open this module has scheduled but not yet performed.
// Leaving an open panel and coming straight back schedules another open while
// the panel is still up; without this the press that follows closes the panel
// and the stale timer springs it open again a moment later.
document.addEventListener('pointerdown', (event) => {
	const wrapper = event.target?.closest?.('[data-trigger="hover"]');
	if (wrapper) clearTimeout(timers.get(wrapper));
}, { capture: true });

document.addEventListener('pointerout', (event) => {
	if (!fine.matches) return;
	const wrapper = crossing(event);
	const parts = wrapper && partsOf(wrapper);
	if (!parts) return;
	schedule(wrapper, delay(wrapper, '--yeti-dropdown-close-delay'), () => {
		// hidePopover throws when the popover is already closed.
		if (!opened.has(parts.panel) || !parts.panel.matches(':popover-open')) return;
		parts.panel.hidePopover();
	});
});
}

// range.js
{
// Range: the filled track and the value readout follow the thumb.
//
// CSS cannot read an input's value, so a range's coloured track has to be told
// how far along it is. Until now the field's docs handed the reader a line to
// paste; this is that line, shipped, and doing the part the pasted one got
// wrong.
//
// What it sets is a unitless share, 0 to 1, and nothing else. The thumb's
// centre does not travel the whole track: it stops half a thumb from each end,
// so a plain percentage and the thumb disagree by that much at the extremes —
// eleven pixels on the field page's own demo. Correcting it needs the thumb's
// width, which CSS knows in its own units and JavaScript would have to measure
// and re-measure on every resize. So the share goes out and field.css does the
// arithmetic, and nothing here reads layout.
//
// The readout is an output element the author puts in the field. Its text is
// the input's value; where it sits is field.css's business. Without the module
// the track simply sits at whatever --yeti-range-value says, which is what a
// page that never loads this keeps doing.
const share = (input) => {
	const min = Number(input.min === '' ? 0 : input.min);
	const max = Number(input.max === '' ? 100 : input.max);
	const span = max - min;
	if (!Number.isFinite(span) || span === 0) return 0;
	return Math.min(1, Math.max(0, (Number(input.value) - min) / span));
};

function sync(input) {
	const field = input.closest('.field');
	// On the field, not the input. Custom properties inherit, so the track
	// still reads it, and the readout beside the input can read it too — a
	// sibling cannot see a property set on the input itself.
	(field || input).style.setProperty('--yeti-range-value', String(share(input)));
	const output = field && field.querySelector(':scope > output');
	// Only the text. A screen reader already hears the value from the input
	// itself, which is why field.css hides this from the accessibility tree.
	if (output) output.textContent = input.value;
}

const syncAll = (root) => root.querySelectorAll?.('.field input[type="range"]').forEach(sync);
syncAll(document);

// Delegated, so a range added after load works too, and one listener covers a
// form of them. input rather than change: the track should follow the drag.
document.addEventListener('input', (event) => {
	const input = event.target?.closest?.('.field input[type="range"]');
	if (input) sync(input);
});

// A range that arrives later starts with its track already filled, rather than
// waiting for the first drag.
new MutationObserver((records) => {
	for (const record of records) for (const node of record.addedNodes) {
		if (node.nodeType !== Node.ELEMENT_NODE) continue;
		if (node.matches('.field input[type="range"]')) sync(node);
		syncAll(node);
	}
}).observe(document.documentElement, { childList: true, subtree: true });
}

// tabs.js
{
// Tabs: pairs every tab with the panel its aria-controls names, shows one of
// them, and moves selection with the arrow keys. Without this module the CSS
// hides nothing, so every panel is readable; loading it is an enhancement.
// Tabs added after load are not picked up.
// A panel may hold tabs of its own, so a root's tabs are only those in its
// own tablist: the first one whose nearest .tabs is this root, never one
// inside a nested .tabs.
const tabsOf = (root) => {
	const list = [...root.querySelectorAll('[role="tablist"]')].find((candidate) => candidate.closest('.tabs') === root);
	return list ? [...list.querySelectorAll('[role="tab"]')] : [];
};

function select(root, tab) {
	for (const other of tabsOf(root)) {
		const on = other === tab;
		other.setAttribute('aria-selected', String(on));
		other.tabIndex = on ? 0 : -1;
		const panel = document.getElementById(other.getAttribute('aria-controls'));
		if (panel) panel.hidden = !on;
		// A panel with nothing focusable inside it must take focus itself, or
		// Tab leaves the tab list and skips straight past the content.
		if (panel) panel.tabIndex = panel.querySelector('a, button, input, select, textarea, [tabindex]') ? -1 : 0;
	}
}

// Selecting is what both handlers do, and only a selection a reader made is
// an event: the pass at load is not a change, it is the markup being obeyed.
function choose(root, tab) {
	select(root, tab);
	tab.focus();
	root.dispatchEvent(new CustomEvent('yeti:select', {
		bubbles: true,
		composed: true,
		detail: { tab, panel: document.getElementById(tab.getAttribute('aria-controls')) },
	}));
}

for (const root of document.querySelectorAll('.tabs')) {
	const tabs = tabsOf(root);
	if (tabs.length) select(root, tabs.find((tab) => tab.getAttribute('aria-selected') === 'true') ?? tabs[0]);
}

// A link elsewhere on the page, or the URL itself, can point at something
// inside a hidden panel. Opening that panel's tab is what makes the target
// reachable at all; a reader who did not touch the tabs did not ask to be
// moved, so this selects and reveals without taking focus the way choose()
// does for an actual tab activation. The target may sit in tabs nested inside
// another tab's panel, so every enclosing panel is opened, walking outward
// from the target, the outermost last; each tab that changes dispatches
// yeti:select. `instant` is true only for the pass at load: the page has not
// been seen yet, so landing on the target should be immediate, not a glide
// the reader watches happen to a page they have not looked at; a later
// hashchange, from a link they just clicked, keeps the page's own scroll
// behavior.
function reveal(instant) {
	const hash = location.hash;
	if (!hash) return;
	let id;
	try {
		id = decodeURIComponent(hash.slice(1));
	} catch {
		return;
	}
	if (!id) return;
	const target = document.getElementById(id);
	if (!target) return;
	let changed = false;
	let panel = target.closest('[role="tabpanel"]');
	while (panel) {
		const root = panel.closest('.tabs');
		if (!root) break;
		const tab = tabsOf(root).find((candidate) => candidate.getAttribute('aria-controls') === panel.id);
		if (tab && tab.getAttribute('aria-selected') !== 'true') {
			select(root, tab);
			changed = true;
			root.dispatchEvent(new CustomEvent('yeti:select', {
				bubbles: true,
				composed: true,
				detail: { tab, panel },
			}));
		}
		panel = root.parentElement?.closest('[role="tabpanel"]');
	}
	if (changed) target.scrollIntoView(instant ? { behavior: 'instant' } : undefined);
}

reveal(true);
window.addEventListener('hashchange', () => reveal(false));

document.addEventListener('click', (event) => {
	// The page's own listener ran first and asked for nothing to happen.
	if (event.defaultPrevented) return;
	const tab = event.target?.closest?.('.tabs [role="tab"]');
	if (!tab) return;
	choose(tab.closest('.tabs'), tab);
});

document.addEventListener('keydown', (event) => {
	// The page's own listener ran first and asked for nothing to happen.
	if (event.defaultPrevented) return;
	const tab = event.target?.closest?.('.tabs [role="tab"]');
	if (!tab) return;
	const root = tab.closest('.tabs');
	const tabs = tabsOf(root);
	const vertical = root.getAttribute('data-orientation') === 'vertical';
	const step = { [vertical ? 'ArrowDown' : 'ArrowRight']: 1, [vertical ? 'ArrowUp' : 'ArrowLeft']: -1 }[event.key];
	let target;
	if (step) target = tabs[(tabs.indexOf(tab) + step + tabs.length) % tabs.length];
	else if (event.key === 'Home') target = tabs[0];
	else if (event.key === 'End') target = tabs[tabs.length - 1];
	if (!target) return;
	event.preventDefault();
	choose(root, target);
});
}

// toc.js
{
// On-page nav: the link whose heading is topmost in view is the current one.
//
// One IntersectionObserver per toc rather than a scroll listener, because the
// browser does the measuring itself and hands over only the headings whose
// visibility actually changed; a scroll handler would run on every frame of
// every scroll to answer the same question.
//
// Without this module the list is a list of links and every one of them still
// works; what is lost is the mark following the reading position. Tocs added
// after load are not picked up, as with tabs.js: the observer is wired once.
for (const toc of document.querySelectorAll('.toc')) {
	const links = [...toc.querySelectorAll('a[href^="#"]')];
	const byHeading = new Map();
	for (const link of links) {
		// decodeURIComponent, because a heading id with a non-ASCII character
		// arrives percent-encoded in the href and getElementById wants the id.
		const heading = document.getElementById(decodeURIComponent(link.getAttribute('href').slice(1)));
		if (heading) byHeading.set(heading, link);
	}
	if (!byHeading.size) continue;

	// Document order, not the order the links happen to be written in: a toc
	// that lists its links out of order should still mark the topmost heading.
	const headings = [...byHeading.keys()].sort((a, b) => (a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING ? -1 : 1));
	const visible = new Set();

	const update = () => {
		const heading = headings.find((h) => visible.has(h));
		// Between two headings nothing is in view. The last mark is still the
		// truest answer there, so it stays rather than flickering off.
		if (!heading) return;
		const link = byHeading.get(heading);
		if (link.getAttribute('aria-current') === 'true') return;
		for (const other of links) other.removeAttribute('aria-current');
		link.setAttribute('aria-current', 'true');
		toc.dispatchEvent(new CustomEvent('yeti:current', { bubbles: true, composed: true, detail: { link, heading } }));
	};

	// Default options on purpose: threshold 0 with no root margin is what makes
	// "topmost in view" mean "any part of the heading visible", which is the rule
	// the headings.find above assumes. A margin or a higher threshold would shift
	// the answer to a heading the reader can see but the observer cannot.
	const observer = new IntersectionObserver((entries) => {
		for (const entry of entries) {
			if (entry.isIntersecting) visible.add(entry.target);
			else visible.delete(entry.target);
		}
		update();
	});
	for (const heading of headings) observer.observe(heading);
}
}

// validate.js
{
// Form validation: the browser already knows what is wrong with a control and
// has a sentence for it in the reader's own language. This puts that sentence
// where the field already shows an error, marks the control the way the field
// already reads, and stops the submit.
//
// The submit is stopped for every invalid control in the form, whether or not
// it sits inside a .field. A form carrying novalidate has no bubble of its
// own, so a control the page left outside a field would otherwise submit an
// invalid form in silence. The field decides only where a message can go.
//
// It runs on submit, never before, so nothing is red while a person is still
// typing their first character; :user-invalid covers the field they have
// finished with, and this covers the one they never touched.
//
// A form that loads this should carry novalidate. Without it the browser's own
// bubble opens on the first invalid control and the submit event is never
// fired at all, so this would never run. The docs say so.
//
// Delegated on document, so a form added after load works too, and safe on a
// page with no form at all. No rules of its own, no async checks, and no
// message catalogue: everything it says comes from the platform.

// The slots this module filled. An author's own message is never overwritten,
// but the browser's own wording is rewritten as the reason changes: "fill in
// this field" becomes "include an @" once something has been typed. A slot the
// page fills in itself after the module has already written to it is treated as
// the module's own and overwritten on the next submit; that is accepted, since
// remembering it was written is what keeps the wording current.
const written = new WeakSet();

function mark(control) {
	control.setAttribute('aria-invalid', 'true');
	// The nearest field that has a slot, not simply the nearest field. A radio
	// or checkbox group is a fieldset.field wrapping one div.field per choice,
	// so the message belongs to the fieldset that carries the [data-error] and
	// not to the bare div around the input, which has none. A control with no
	// such field above it is still marked; there is just nowhere to write.
	const error = control.closest('.field:has(> [data-error])')?.querySelector(':scope > [data-error]');
	if (!error) return;
	if (written.has(error) || !error.textContent.trim()) {
		error.textContent = control.validationMessage;
		written.add(error);
	}
}

document.addEventListener('submit', (event) => {
	const form = event.target;
	if (!form?.matches?.('form')) return;
	// The page's own listener ran first and asked for nothing to happen.
	if (event.defaultPrevented) return;
	// validity rather than checkValidity(), which fires an invalid event of
	// its own on every control it touches; the only event this module should
	// be dispatching is its own. Every invalid control counts, field or no
	// field, so nothing invalid slips past under novalidate.
	const controls = [...form.elements].filter((el) => el.willValidate && !el.validity.valid);
	if (!controls.length) return;
	event.preventDefault();
	for (const control of controls) mark(control);
	// The first one, because that is where the reader has to start, and
	// focusing it is also what scrolls it into view.
	controls[0].focus();
	form.dispatchEvent(new CustomEvent('yeti:invalid', { bubbles: true, composed: true, detail: { controls } }));
});

// input for what is typed, change for what is picked. The mark goes the moment
// the control is valid again, rather than waiting for the next submit, because
// a red box that stays red after it has been fixed teaches a reader to ignore
// the color.
const clear = (event) => {
	const control = event.target;
	if (!control?.willValidate) return;
	// change fires only on the radio that became checked, but every radio of
	// the group was marked and the group's error shows while any one of them
	// still carries the mark, so the whole group is cleared together. A shared
	// name gives a RadioNodeList from form.elements; a name held by one control
	// gives that control, which has a nodeType where the list does not.
	const named = control.name && control.form ? control.form.elements[control.name] : null;
	const group = !named ? [control] : named.nodeType ? [named] : [...named];
	for (const member of group) {
		if (member.willValidate && member.validity.valid) member.removeAttribute('aria-invalid');
	}
};
document.addEventListener('input', clear);
document.addEventListener('change', clear);
}
