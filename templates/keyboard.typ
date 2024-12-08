// thank you keyle :)
#let button(content, width, url: none) = {
  let text-color = black
  let stroke-color = rgb("#555")
  let bg-color = rgb("#fff")

  let content = pad(y: 4pt, content)

  let cust-rect = rect.with(
    width: width,
    stroke: stroke-color,
    fill: bg-color,
  )

  let button = cust-rect(
    align(horizon, text(fill: text-color, content)),
  )
  let shadow = cust-rect(
    outset: 2.2pt,
    fill: bg-color,
    stroke: stroke-color + 1.2pt,
    align(horizon, text(fill: text-color, content)),
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

#let key(content, url) = {
  let text-color = white
  let bg-color = rgb("#333")
  let stroke-color = rgb("#2b2b2b")

  let cust-rect = rect.with(
    width: 36pt,
    height: 36pt,
    stroke: bg-color,
    fill: stroke-color,
    radius: 50%,
  )

  let button = cust-rect(
    align(center + horizon, text(fill: white, content)),
  )
  let shadow = cust-rect(
    outset: 2.2pt,
    fill: white,
    stroke: stroke-color + 1.2pt,
    align(center + horizon, text(fill: white, content)),
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
      ..chars.map(x => key(text(size: 28pt, upper(x)), url + x)),
      ""
    )
  }

  row(url, chars: "qwertyuiop".codepoints(), left-pad: 1fr, right-pad: 1fr)
  row(url, chars: "asdfghjkl".codepoints(), left-pad: 0.25fr, right-pad: 0.25fr)
  row(url, chars: "zxcvbnm".codepoints(), left-pad: 0.5fr, right-pad: 0.75fr)
}

#let input_and_keyboard(title, input, base-url) = {
  align(center, {
    align(left, {        
      v(4%)
  
      text(size: 18pt, fill: luma(80))[
        #h(5.5pt)
        #title
      ]
  
      v(-3%)
  
      grid(
        inset: 8pt,
        columns: (1fr, auto),
        button(text(font: "New Computer Modern", size: 24pt, input + strong[$bracket.b$]), 100%),
        align(horizon, box(key(align(center, text(size: 24pt, $arrow.l.hook$)), base-url + "next")))
      )
  
      keyboard(base-url)
    })
  })

}
