#set text(size: 10pt, weight: "regular", style: "normal")

#let url = "https://pdforum.meow.i.ng"

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

#let navbar = box(width: 100%, inset: 0.2em, stroke: black)[
  #box(width: 100%, inset: 0.35em, stroke: black)[
    #align(center)[
      #show link: underline
      
      #smallcaps(lower(link(url + "/home")[HOME] + h(5%) + link(url + "/browse")[BROWSE] + h(5%) + link(url + "/announcements")[ANNOUNCEMENTS] + h(5%) + link(url + "/account")[ACCOUNT] + h(5%) + link(url + "/settings")[SETTINGS]))
    ]
    #v(0.15em)
  ]
]

#let brand = grid(
  columns: (25%, 1fr, 25%),
  align(center)[
    #text(size: 2.3em)[
      #par(leading: 0.2em)[
        HTML-Free!
      ]
    ]
  ],
  align(center)[
    #text(size: 5em)[
      PDForum
    ]
  ],
  align(center + horizon)[
    #box(stroke: black, inset: 0.4em, width: 50%, outset: -5%)[
      #par(leading: 0.2em)[
        #show link: underline
        
        #smallcaps(link("https://github.com/goldenstack/pdforum/")[source])

        #v(-0.8em)
        
        #smallcaps[made by #link("https://goldenstack.net/")[golden]]

        #v(0.1em)
      ]
    ]
  ]
)

#let info = box(width: 100%, inset: 0.25em, stroke: (top: black, bottom: black))[
  #box(width: 100%, inset: 0.5em, stroke: (top: black, bottom: black))[
    #box(width: 1fr)[
      #align(left)[
        #h(2em)
        #smallcaps(lower("Web A-1a"))
      ]
    ]
    #box(width: 1fr)[   
      #align(center)[
        #smallcaps(lower("Thursday, October 24th, 2024"))
      ]
    ]
    #box(width: 1fr)[
      #align(right)[
        #smallcaps(lower("PDF edition"))
        #h(2em)
      ] 
    ]
  ]
]

#let header = navbar + brand + info

#place(top, float: true, scope: "parent")[
  #show: header
]

#let post(author: none, id: none, time: none, content: none) = link(url + "/post/" + id, strong(author)) + h(0.5em) + text(size: 9pt, fill: luma(100), smallcaps[Author]) + h(0.5em) + sym.dot + h(0.5em) + smallcaps(text(fill: luma(100), link(url + "/post/" + id, time))) + v(0.25em) + content + v(1em)

#{
  let data = read("data.txt").split("\n")
  
  for i in range(data.len(), step: 4) {
    post(author: data.at(i), id: data.at(i + 1), time: data.at(i + 2), content: data.at(i + 3))
  }
}

