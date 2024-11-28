#import "header.typ": *
#import "keyboard.typ": *

#set text(size: 10pt, weight: "regular", style: "normal")

#let info = yaml("info.yml")

#set page("a4",
  margin:(top: 2cm,
  bottom: 2cm,
  left: 2.5cm, right: 2cm))
  
#set page(footer: context [
  #set text(size: 8pt)
  #line(length: 100%,stroke: 0.2pt + gray)
  PDForum - 2024-10-24
  #h(1fr)
  #counter(page).display(
    "1/1",
    both: true,
  )
])

#place(top, float: true, scope: "parent")[
  #show: header(info.url, info.auth)
]

#let data = read("data.txt")

#align(center, {
  align(left, {
    v(4%)
    
    text(size: 18pt, fill: luma(80))[
      #h(5.5pt)
      ALIAS
    ]

    v(-3%)

    grid(
      inset: 8pt,
      columns: (1fr, auto),
      button(text(font: "New Computer Modern", size: 24pt, data + strong[$bracket.b$]), 100%),
      key(align(center, text(size: 24pt, $arrow.l.hook$)), info.url + "/")
    )

    keyboard(info.url + "/register/")
  })
})