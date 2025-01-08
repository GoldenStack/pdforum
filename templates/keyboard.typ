// thank you keyle :)
#let text-box(content, width, url: none, stroke-color: rgb("#555"), bg-color: rgb("#fff")) = {
  let content = pad(y: 4pt, content)

  let cust-rect = rect.with(
    width: width,
    stroke: stroke-color,
    fill: bg-color,
  )

  let button = cust-rect(
    align(horizon, content),
  )
  let shadow = cust-rect(
    outset: 2.2pt,
    fill: bg-color,
    stroke: stroke-color + 1.2pt,
    align(horizon, content),
  )
  {
    place(shadow)
    if url == none {
      button
    } else {
      link(url, button)
    }
  }
}

#let key(content, url, bg-color: rgb("#2b2b2b"), stroke-color: rgb("#333")) = {
  let cust-rect = rect.with(
    width: 36pt,
    height: 36pt,
    stroke: stroke-color,
    fill: bg-color,
    radius: 50%,
  )

  let text = align(center + horizon, content);

  let button = cust-rect(text)
  
  let shadow = cust-rect(
    outset: 2.2pt,
    fill: white,
    stroke: stroke-color + 1.2pt,
    text,
  )
  {
    place(shadow)
    link(url, button)
  }
}

#let keyboard(url) = {
  let row(url, chars: (), left-pad: 0fr, right-pad: 0fr) = {
    grid(
      columns: (left-pad, ..chars.map(x => auto), right-pad),
      inset: 5pt,
      align: left,
      "",
      ..chars.map(x => key(text(size: 28pt, fill: white, upper(x)), url + x)),
      ""
    )
  }

  row(url, chars: "qwertyuiop".codepoints(), left-pad: 1fr, right-pad: 1fr)
  row(url, chars: "asdfghjkl".codepoints(), left-pad: 0.25fr, right-pad: 0.25fr)
  row(url, chars: "zxcvbnm".codepoints(), left-pad: 0.5fr, right-pad: 0.75fr)
}

#let text-box-next(content, base-url, selected: true) = {
  if selected {
    content += strong[$bracket.b$]
  }
  
  let content = text(font: "New Computer Modern", size: 24pt, content)
  
  let content-box = if selected {
    text-box(content, 100%)
  } else {
    text-box(stroke-color: luma(160), text(fill: luma(180), content), 100%)
  }

  let next-box = if selected {
    align(horizon, box(key(align(center, text(size: 24pt, fill: white, $arrow.l.hook$)), base-url + "next")))
  } else {
    // key(bg-color: luma(220), stroke-color: luma(190), align(center, text(size: 24pt, fill: white, $arrow.l.hook$)), base-url + "next")
    v(36pt)
  }
  
  grid(
    column-gutter: 10pt,
    columns: (1fr, 52pt),
    content-box, next-box
  )
}