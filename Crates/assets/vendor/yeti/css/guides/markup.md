---
raw: true
title: "Why the markup looks like this"
description: "Classes name the thing, data-* attributes configure it. The alternatives we tried, custom elements, bare attributes and short prefixes, and why each lost."
nav_group: "Guides"
nav_order: 12
---

# Why the markup looks like this

<p class="lede">Yeti markup has two parts: a class that names what the element is, and <code>data-*</code> attributes that configure it. <code>&lt;div class="columns" data-threshold="md"&gt;</code>. That shape was questioned before the release, seriously, because the <code>data-</code> prefix is noisy and markup should be pleasant to read. This page records what we looked at and why each alternative lost. If you have wondered why Yeti doesn't use custom elements, or why it doesn't drop the prefix, the answer is here.</p>

## The starting point

Here is a typical piece of Yeti markup:

```html
<div class="columns" data-threshold="md">
	<section class="box" data-surface="raised" data-border>…</section>
	<section class="box" data-surface="raised" data-border>…</section>
	<section class="box" data-surface="raised" data-border>…</section>
</div>
```

Under the surface, every attribute value sets a private custom property once, and the layout or component reads that property. `[data-gap="md"]` sets `--_yeti-gap`, and `.stack` reads `--_yeti-gap`. Every alternative below keeps that engine. What was in question was only the surface: what you type.

## Custom elements without JavaScript

A tag name can be styled even when no script ever defines it. `<y-stack gap="lg">` is an unknown element to the browser, but CSS can select `y-stack` and `[gap="lg"]` like anything else. There is no component bundle, no upgrade step and no shadow DOM, and an autonomous custom element may carry any attribute and still be valid HTML. Every Layout's primitives, `<stack-l>` and the rest, take the same shape, though they use JavaScript.

It reads beautifully. It lost for three reasons.

**Layouts belong on semantic elements.** A cluster is very often a list, and a stack is very often a form:

```html
<ul class="cluster">
	<li><a href="/plan">Plan</a></li>
	<li><a href="/build">Build</a></li>
	<li><a href="/ship">Ship</a></li>
</ul>
<form class="stack">
	<label for="email">Email</label>
	<input id="email" type="email">
	<button type="submit">Subscribe</button>
</form>
```

A tag can't be added to an element the way a class can. `<y-cluster><ul>…</ul></y-cluster>` breaks the layout, because the list items are no longer the flex children. `<y-cluster role="list">` rebuilds a `<ul>`, worse. The usual escape hatch is to accept both the tag and the class, which gives every layout two spellings, two sets of docs and two sets of tests.

**Interactive components must stay native.** A button is a `<button>`, a dialog a `<dialog>`, an accordion a `<details>`. An unknown element has no role, no focus and no form behaviour, and getting those back means JavaScript, which defeats the point. So components keep their classes whatever layouts do, and the framework would speak two dialects.

**Markers live on native children.** `data-span` sits on a grid's `<li>`, and `data-space` on a stack's `<h2>`. Bare attributes are only valid on custom elements, so the children would keep the prefix: `<y-grid columns="3"><li data-span="2">`. That's two dialects on one line.

## Bare attributes

Keep the classes, drop the prefix:

```html
<div class="columns" threshold="md">
	<section class="box" surface="raised" border>…</section>
	<section class="box" surface="raised" border>…</section>
	<section class="box" surface="raised" border>…</section>
</div>
```

This is the most readable option by a distance, and it works in every browser: unknown attributes stay in the page and CSS selects them. Alpine, htmx, Vue and Angular have all relied on that for years. The costs are real, though.

**Some names already mean something.** HTML has used these attributes for decades, and on the right element the browser still acts on them:

| Attribute | What the browser does with it |
| --- | --- |
| `align` | On `div`, `p` and headings, it aligns the text. |
| `border` | On `table` and `img`, it draws a border. Yeti's own `table` takes a border attribute. |
| `width`, `height` | Resizes images, video, canvas, iframes and tables. |
| `size` | Changes the width of `input` and `select`. |
| `min`, `max` | Constrain `input` and `meter`. |
| `nowrap` | Stops a table cell from wrapping. |
| `fill` | Paints SVG, so icons would be affected. |

About fourteen of Yeti's attribute names are already HTML attributes somewhere; the table lists the ones that bite. They would all need new names, and those names would be worse words for the same ideas.

**Sanitizers strip them.** Rich-text sanitizers such as DOMPurify commonly let every `data-*` attribute through and remove attributes they don't recognize. Yeti markup pasted into a CMS field would silently lose `threshold` and keep `data-threshold`.

**Validators flag them**, and HTML may one day add a global attribute that takes one of the names. Both are small costs, but they are real.

Bare attributes would mean trading correctness for looks in exactly the places the problems are hardest to notice: a table with an unexpected border, a layout that quietly stops working after a trip through a CMS.

## A prefix without the dash

If the dash is what hurts, drop the dash but keep a marker: `ythreshold`, `ysurface`, `yborder`.

That fixes collisions completely, since no HTML attribute starts with `y` and a word. It doesn't fix the rest. A sanitizer strips `ythreshold` as readily as `threshold`, and a validator flags it the same way. To HTML, `datathreshold` is no different either; only `data-`, with the dash, is the valid prefix.

It also adds a new problem. Anyone who writes CSS, SVG or canvas reads `y` as the vertical axis. `ygap` looks like a row gap, `yalign` like vertical alignment, and `yscroll` like vertical scrolling. Most of Yeti's layout vocabulary would read as something it isn't.

## A short prefix with the dash

`yt-threshold`, `yt-surface`, `yt-border`. This is the htmx model, and it suits htmx, whose attributes are commands. Set beside `data-` it is three characters shorter and no different in shape: the dash is back, and it was the dash that hurt. It is also invalid where `data-` is valid and stripped where `data-` survives, and `yt` reads as YouTube before it reads as Yeti.

## Where that left us

Each step ended in the same place. Every prefix keeps the cost of having a prefix and adds the cost of being non-standard. That leaves two coherent positions:

1. **`data-*`.** Valid everywhere, survives sanitizers, can't collide, and costs a little visual noise.
2. **Bare words**, with the colliding names renamed. The cleanest to read, at the cost of validation, sanitizer survival and some good names.

Yeti takes the first. Markup that keeps working after it has been pasted, sanitized, validated and handed to a browser that knows thirty years of HTML is worth five characters an attribute.

Some of the readability is won back elsewhere. The attribute names are plain words with short, fixed value lists (`md`, `raised`, `high`), and the same few names (`data-gap`, `data-variant`, `data-emphasis`, `data-size`) mean the same thing on every element that takes them. Editors complete them from the package, so the prefix is typed once and then completed for you; the [installation guide](install.md) has the setting.

## What would change our minds

If a future HTML adds a sanctioned way to name custom attributes on native elements without the `data-` prefix, the case against bare words falls apart, and this page will be rewritten. Until then, the class names the thing and `data-*` configures it.
