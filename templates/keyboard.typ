// thank you keyle :)
#let user-input(content) = {
  let stroke-color = rgb("#555")
  let fill-color = rgb("#fff")
  let layers = 2

  content = align(horizon, content)
  
  let cust-rect = rect.with(
    inset: (x: 10pt, y: 10pt),
    width: 100%,
    stroke: stroke-color + 0.6pt,
    radius: 2pt,
    fill: fill-color,
  )
  let button = cust-rect(
    text(fill: black, content),
  )
  let shadow = cust-rect(
    fill: stroke-color,
    text(fill: fill-color, content),
  )
  for n in range(layers) {
    place(dx: 0.6pt * n, dy: 0.6pt * n, shadow)
  }
  button
}

#let button(content, width) = {
  let stroke-color = rgb("#555")
  let fill-color = rgb("#eee")
  let layers = 2

  content = align(center + horizon, content)
  
  let cust-rect = rect.with(
    inset: (x: 10pt, y: 10pt),
    width: width,
    stroke: stroke-color + 0.6pt,
    radius: 2pt,
    fill: fill-color,
  )
  let button = cust-rect(
    text(fill: black, content),
  )
  let shadow = cust-rect(
    fill: stroke-color,
    text(fill: fill-color, content),
  )
  for n in range(layers) {
    place(dx: 0.6pt * n, dy: 0.6pt * n, shadow)
  }
  button
}

#let key(content) = button(content, 40pt)

#let keyboard(url) = {
  let row(url, chars: (), left-pad: 0fr, right-pad: 0fr) = {
    grid(
      columns: (left-pad, ..chars.map(x => auto), right-pad),
      inset: 5pt,
      align: left,
      "",
      ..chars.map(t => text(size: 36pt, t)).map(key).zip(chars).map(pair => link(url + pair.at(1), upper(pair.at(0))))
    )
  }

  row(url, chars: "qwertyuiop".codepoints())
  row(url, chars: "asdfghjkl".codepoints(), left-pad: 0.25fr, right-pad: 0.25fr)
  row(url, chars: "zxcvbnm".codepoints(), left-pad: 0.5fr, right-pad: 0.75fr)
}