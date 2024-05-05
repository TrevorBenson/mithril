"use strict";(self.webpackChunkmithril_doc=self.webpackChunkmithril_doc||[]).push([[2057],{74314:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>d,contentTitle:()=>a,default:()=>h,frontMatter:()=>s,metadata:()=>o,toc:()=>l});var i=n(74848),r=n(28453);const s={slug:5,title:"5. Use rfc3339 for date formatting \n",authors:[{name:"Mithril Team"}],tags:["Accepted"],date:new Date("2023-06-21T00:00:00.000Z")},a=void 0,o={permalink:"/doc/adr/5",source:"@site/adr/005-use-rfc3339-for-dates.md",title:"5. Use rfc3339 for date formatting \n",description:"Status",date:"2023-06-21T00:00:00.000Z",formattedDate:"June 21, 2023",tags:[{label:"Accepted",permalink:"/doc/adr/tags/accepted"}],readingTime:1.18,hasTruncateMarker:!1,authors:[{name:"Mithril Team"}],frontMatter:{slug:"5",title:"5. Use rfc3339 for date formatting \n",authors:[{name:"Mithril Team"}],tags:["Accepted"],date:"2023-06-21T00:00:00.000Z"},unlisted:!1,prevItem:{title:"6. Errors implementation Standard\n",permalink:"/doc/adr/6"},nextItem:{title:"4. Mithril Network Upgrade Strategy\n",permalink:"/doc/adr/4"}},d={authorsImageUrls:[void 0]},l=[{value:"Status",id:"status",level:2},{value:"Context",id:"context",level:2},{value:"Decision",id:"decision",level:2},{value:"Consequences",id:"consequences",level:2}];function c(e){const t={a:"a",code:"code",em:"em",h2:"h2",li:"li",p:"p",pre:"pre",strong:"strong",ul:"ul",...(0,r.R)(),...e.components};return(0,i.jsxs)(i.Fragment,{children:[(0,i.jsx)(t.h2,{id:"status",children:"Status"}),"\n",(0,i.jsx)(t.p,{children:"Accepted"}),"\n",(0,i.jsx)(t.h2,{id:"context",children:"Context"}),"\n",(0,i.jsx)(t.p,{children:"Previously, on the Mithril project we did not have a preferred format for the dates in our applications, leading to\nmultiple formats being used."}),"\n",(0,i.jsxs)(t.p,{children:["For example when querying a certificate from an aggregator, the ",(0,i.jsx)(t.code,{children:"initiated_at"})," field did not specify the timezone,\ntimezone that could be found in the ",(0,i.jsx)(t.code,{children:"sealed_at"})," field:"]}),"\n",(0,i.jsx)(t.pre,{children:(0,i.jsx)(t.code,{className:"language-json",children:'{\n  "initiated_at": "2023-05-26T00:02:23",\n  "sealed_at": "2023-05-26T00:03:23.998753492Z"\n}\n'})}),"\n",(0,i.jsxs)(t.p,{children:["Same problem in our databases where a date could be stored without timezone and milliseconds (ie: ",(0,i.jsx)(t.code,{children:"2023-06-13 16:35:28"}),")\nin one table column and with them in another (ie: ",(0,i.jsx)(t.code,{children:"2023-06-13T16:35:28.143292875Z"}),")."]}),"\n",(0,i.jsxs)(t.p,{children:["The ",(0,i.jsx)(t.a,{href:"https://datatracker.ietf.org/doc/html/rfc3339",children:"RFC 3339"})," is a widely used, easily readable, mostly numeric (no\ntranslation is needed to parse the day or the month), format. Also, it always includes the timezone meaning that our\nclient can convert such date to their local time if needed."]}),"\n",(0,i.jsx)(t.h2,{id:"decision",children:"Decision"}),"\n",(0,i.jsx)(t.p,{children:(0,i.jsx)(t.em,{children:"Therefore"})}),"\n",(0,i.jsxs)(t.ul,{children:["\n",(0,i.jsxs)(t.li,{children:["We commit to use ",(0,i.jsx)(t.strong,{children:"RFC 3339"})," compatible date and time whenever we need to store or show a date and time."]}),"\n"]}),"\n",(0,i.jsx)(t.h2,{id:"consequences",children:"Consequences"}),"\n",(0,i.jsxs)(t.ul,{children:["\n",(0,i.jsxs)(t.li,{children:["All dates and time must use a dedicated type in the application, ie: the ",(0,i.jsx)(t.code,{children:"DateTime<Utc>"})," type from\n",(0,i.jsx)(t.a,{href:"https://crates.io/crates/chrono",children:"chrono"})," crate.","\n",(0,i.jsxs)(t.ul,{children:["\n",(0,i.jsxs)(t.li,{children:["This means that dates must ",(0,i.jsx)(t.strong,{children:"never"})," be stored in our types using Strings."]}),"\n"]}),"\n"]}),"\n",(0,i.jsxs)(t.li,{children:["Internally, we will always use the ",(0,i.jsx)(t.strong,{children:"UTC timezone"}),", to avoid useless conversions between timezones."]}),"\n",(0,i.jsx)(t.li,{children:"Users or scripts querying dates from our applications or from our databases will be able to parse all of them using\nthe same format."}),"\n"]})]})}function h(e={}){const{wrapper:t}={...(0,r.R)(),...e.components};return t?(0,i.jsx)(t,{...e,children:(0,i.jsx)(c,{...e})}):c(e)}},28453:(e,t,n)=>{n.d(t,{R:()=>a,x:()=>o});var i=n(96540);const r={},s=i.createContext(r);function a(e){const t=i.useContext(s);return i.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function o(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(r):e.components||r:a(e.components),i.createElement(s.Provider,{value:t},e.children)}}}]);