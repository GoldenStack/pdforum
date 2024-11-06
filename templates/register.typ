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
    v(5%)
    text(size: 24pt)[
      #h(8pt)
      Username
    ]

    v(-2%)
    
    grid(
      inset: 8pt,
      columns: (1fr, auto),
      user-input(text(size: 24pt, data + strong[|])),
      link(info.url + "", button(text(size: 24pt, "continue"), auto))
    )

    keyboard(info.url + "/register/")
  })
})
