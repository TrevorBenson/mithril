"use strict";(self.webpackChunkmithril_doc=self.webpackChunkmithril_doc||[]).push([[8257],{94447:(e,t,i)=>{i.r(t),i.d(t,{assets:()=>h,contentTitle:()=>l,default:()=>d,frontMatter:()=>o,metadata:()=>s,toc:()=>a});var r=i(74848),n=i(28453);const o={sidebar_position:1,sidebar_label:"Architecture"},l="Mithril network architecture",s={id:"mithril/mithril-network/architecture",title:"Mithril network architecture",description:"* The current Mithril network relies on a single aggregator. However, the team is working on a more decentralized version, where multiple aggregators will operate within the same Mithril network.",source:"@site/root/mithril/mithril-network/architecture.md",sourceDirName:"mithril/mithril-network",slug:"/mithril/mithril-network/architecture",permalink:"/doc/next/mithril/mithril-network/architecture",draft:!1,unlisted:!1,editUrl:"https://github.com/input-output-hk/mithril/edit/main/docs/website/root/mithril/mithril-network/architecture.md",tags:[],version:"current",sidebarPosition:1,frontMatter:{sidebar_position:1,sidebar_label:"Architecture"},sidebar:"mithrilSideBar",previous:{title:"Mithril Network",permalink:"/doc/next/category/mithril-network"},next:{title:"Mithril aggregator",permalink:"/doc/next/mithril/mithril-network/aggregator"}},h={},a=[{value:"Architecture diagram",id:"architecture-diagram",level:2},{value:"Mithril nodes",id:"mithril-nodes",level:2}];function c(e){const t={a:"a",admonition:"admonition",blockquote:"blockquote",h1:"h1",h2:"h2",img:"img",li:"li",p:"p",strong:"strong",ul:"ul",...(0,n.R)(),...e.components};return(0,r.jsxs)(r.Fragment,{children:[(0,r.jsx)(t.h1,{id:"mithril-network-architecture",children:"Mithril network architecture"}),"\n",(0,r.jsx)(t.admonition,{type:"info",children:(0,r.jsxs)(t.ul,{children:["\n",(0,r.jsx)(t.li,{children:"The current Mithril network relies on a single aggregator. However, the team is working on a more decentralized version, where multiple aggregators will operate within the same Mithril network."}),"\n"]})}),"\n",(0,r.jsx)(t.h2,{id:"architecture-diagram",children:"Architecture diagram"}),"\n",(0,r.jsx)(t.p,{children:(0,r.jsx)(t.a,{target:"_blank","data-noBrokenLinkCheck":!0,href:i(80569).A+"",children:(0,r.jsx)(t.img,{alt:"Architecture",src:i(26149).A+"",width:"1635",height:"1341"})})}),"\n",(0,r.jsx)(t.h2,{id:"mithril-nodes",children:"Mithril nodes"}),"\n",(0,r.jsx)(t.p,{children:"The network is composed of the following nodes:"}),"\n",(0,r.jsxs)(t.ul,{children:["\n",(0,r.jsxs)(t.li,{children:[(0,r.jsx)(t.a,{href:"/doc/next/mithril/mithril-network/aggregator",children:(0,r.jsx)(t.strong,{children:"Mithril aggregator"})}),":"]}),"\n"]}),"\n",(0,r.jsxs)(t.blockquote,{children:["\n",(0,r.jsx)(t.p,{children:"The trustless node that orchestrates the work of the Mithril signer nodes, gathering their individual signatures to produce Mithril multi-signatures and their associated certificates. The aggregator is also in charge of creating and storing the ledger state snapshot archives."}),"\n"]}),"\n",(0,r.jsxs)(t.ul,{children:["\n",(0,r.jsxs)(t.li,{children:[(0,r.jsx)(t.a,{href:"/doc/next/mithril/mithril-network/signer",children:(0,r.jsx)(t.strong,{children:"Mithril signer"})}),":"]}),"\n"]}),"\n",(0,r.jsxs)(t.blockquote,{children:["\n",(0,r.jsx)(t.p,{children:"The node that works transparently on top of the stake pool operator's Cardano node and individually signs the Cardano chain state."}),"\n"]}),"\n",(0,r.jsxs)(t.ul,{children:["\n",(0,r.jsxs)(t.li,{children:[(0,r.jsx)(t.a,{href:"/doc/next/mithril/mithril-network/client",children:(0,r.jsx)(t.strong,{children:"Mithril client"})}),":"]}),"\n"]}),"\n",(0,r.jsxs)(t.blockquote,{children:["\n",(0,r.jsx)(t.p,{children:"The node used to restore artifacts produced by a Mithril aggregator. It then employs Mithril cryptographic primitives to verify their authenticity and validity."}),"\n"]}),"\n",(0,r.jsxs)(t.ul,{children:["\n",(0,r.jsxs)(t.li,{children:[(0,r.jsx)(t.strong,{children:"Mithril relay"}),":"]}),"\n"]}),"\n",(0,r.jsxs)(t.blockquote,{children:["\n",(0,r.jsxs)(t.p,{children:["A forward proxy that is used to secure communications between the Mithril signer and the Mithril aggregator. More information is available in the ",(0,r.jsx)(t.a,{href:"../../manual/getting-started/run-signer-node#mithril-signer-deployment-model",children:"Mithril signer deployment model"})," section."]}),"\n"]}),"\n",(0,r.jsx)(t.admonition,{type:"tip",children:(0,r.jsxs)(t.p,{children:["For more information about the Mithril protocol, read ",(0,r.jsx)(t.a,{href:"/doc/next/mithril/mithril-protocol/protocol",children:"about Mithril"}),"."]})})]})}function d(e={}){const{wrapper:t}={...(0,n.R)(),...e.components};return t?(0,r.jsx)(t,{...e,children:(0,r.jsx)(c,{...e})}):c(e)}},80569:(e,t,i)=>{i.d(t,{A:()=>r});const r=i.p+"assets/files/architecture-e6fad1720a863c9a3986400cb5ac1287.svg"},26149:(e,t,i)=>{i.d(t,{A:()=>r});const r=i.p+"assets/images/architecture-e6fad1720a863c9a3986400cb5ac1287.svg"},28453:(e,t,i)=>{i.d(t,{R:()=>l,x:()=>s});var r=i(96540);const n={},o=r.createContext(n);function l(e){const t=r.useContext(o);return r.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function s(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(n):e.components||n:l(e.components),r.createElement(o.Provider,{value:t},e.children)}}}]);