#import "header.typ": *

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

#set page(columns: 2)

#place(top, float: true, scope: "parent")[
  #show: header(info.url, info.auth)
]

#let post(author: none, id: none, time: none, content: none) = link(info.url + "/post/" + id, strong(author)) + h(0.5em) + text(size: 9pt, fill: luma(100), smallcaps[Author]) + h(0.5em) + sym.dot + h(0.5em) + smallcaps(text(fill: luma(100), link(info.url + "/post/" + id, time))) + v(0.25em) + content + v(1em)

#{
  let data = read("data.txt").split("\n")
  
  for i in range(data.len(), step: 4) {
    post(author: data.at(i), id: data.at(i + 1), time: data.at(i + 2), content: data.at(i + 3))
  }
}
