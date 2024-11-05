#let navbar(url, signed-in) = box(width: 100%, inset: 0.2em, stroke: black)[
    #box(width: 100%, inset: 0.35em, stroke: black)[
    #align(center)[
      #show link: underline
      
      #smallcaps(lower(link(url + "/")[HOME] + h(5%) + link(url + "/browse")[BROWSE] + h(5%) + link(url + "/announcements")[ANNOUNCEMENTS] + h(5%) + {
        if signed-in {
          link(url + "/account")[ACCOUNT] + h(5%) + link(url + "/settings")[SETTINGS]
        } else {
          link(url + "/login")[LOGIN] + h(5%) + link(url + "/register")[REGISTER]
        }
      }))
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
        #smallcaps(lower(datetime.today().display("[weekday], [month repr:long] [day padding:none], [year]")))
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

#let header(url, signed-in) = navbar(url, signed-in) + brand + info