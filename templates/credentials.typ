#import "common.typ": *
#import "keyboard.typ": *

#let info = yaml("info.yml")

#show: common.with(info)

#let data = read("data.txt")

#align(center, {
  align(left, {

    let username = info.field == "username";
    
    v(4%)

    text(size: 18pt, fill: luma(80))[
      #h(5.5pt)
      #{
        if username {
          "ALIAS"
        } else {
          "PASSCODE"
        }
      }
    ]

    v(-3%)

    grid(
      inset: 8pt,
      columns: (1fr, auto),
      button(text(font: "New Computer Modern", size: 24pt, data.clusters().map(c => if username { c } else { $dot$ }).join(sym.zws) + strong[$bracket.b$]), 100%),
      align(horizon, box(key(align(center, text(size: 24pt, $arrow.l.hook$)), info.url + info.path + "next")))
    )

    keyboard(info.url + info.path)
  })
})
