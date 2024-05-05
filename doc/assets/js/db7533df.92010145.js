"use strict";(self.webpackChunkmithril_doc=self.webpackChunkmithril_doc||[]).push([[3745],{12393:(e,t,i)=>{i.r(t),i.d(t,{assets:()=>c,contentTitle:()=>o,default:()=>d,frontMatter:()=>s,metadata:()=>a,toc:()=>h});var n=i(74848),r=i(28453);const s={title:"Signers list computation in Certificates",authors:[{name:"Mithril Team"}],tags:["certificate"]},o=void 0,a={permalink:"/doc/dev-blog/2022/09/12/certificate-signers-list",source:"@site/blog/2022-09-12-certificate-signers-list.md",title:"Signers list computation in Certificates",description:"The way the Signers list is computed inside a Certificate on the Mithril Aggregator is changing",date:"2022-09-12T00:00:00.000Z",formattedDate:"September 12, 2022",tags:[{label:"certificate",permalink:"/doc/dev-blog/tags/certificate"}],readingTime:.825,hasTruncateMarker:!1,authors:[{name:"Mithril Team"}],frontMatter:{title:"Signers list computation in Certificates",authors:[{name:"Mithril Team"}],tags:["certificate"]},unlisted:!1,prevItem:{title:"Stake Distribution retrieval fixed",permalink:"/doc/dev-blog/2022/09/13/stake-distribution-retrieval"},nextItem:{title:"Genesis Certificate support added",permalink:"/doc/dev-blog/2022/09/07/genesis-certificate-feature"}},c={authorsImageUrls:[void 0]},h=[{value:"The way the Signers list is computed inside a Certificate on the Mithril Aggregator is changing",id:"the-way-the-signers-list-is-computed-inside-a-certificate-on-the-mithril-aggregator-is-changing",level:3}];function l(e){const t={a:"a",code:"code",h3:"h3",p:"p",strong:"strong",...(0,r.R)(),...e.components};return(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(t.h3,{id:"the-way-the-signers-list-is-computed-inside-a-certificate-on-the-mithril-aggregator-is-changing",children:"The way the Signers list is computed inside a Certificate on the Mithril Aggregator is changing"}),"\n",(0,n.jsxs)(t.p,{children:[(0,n.jsx)(t.strong,{children:"PR"}),": ",(0,n.jsx)(t.code,{children:"Implement filtered Signers in Certificate"})," ",(0,n.jsx)(t.a,{href:"https://github.com/input-output-hk/mithril/pull/494",children:"#494"})]}),"\n",(0,n.jsxs)(t.p,{children:[(0,n.jsx)(t.strong,{children:"Issue"}),": ",(0,n.jsx)(t.code,{children:"Record 'contributing' Signers only in Certificate"})," ",(0,n.jsx)(t.a,{href:"https://github.com/input-output-hk/mithril/issues/495",children:"#495"})]}),"\n",(0,n.jsxs)(t.p,{children:["Before this change, the list of Signers displayed in the ",(0,n.jsx)(t.code,{children:"Certificate"})," detail of the ",(0,n.jsx)(t.a,{href:"https://mithril.network/explorer/",children:"Mithril Explorer"})," was the list of ",(0,n.jsx)(t.strong,{children:"all eligible"})," Signers of the epoch used for signing (those who have successfully registered with the Mithril Aggregator ",(0,n.jsx)(t.code,{children:"2"})," epochs earlier)."]}),"\n",(0,n.jsxs)(t.p,{children:["Now that this change has been merged, the list of Signers displayed will only include the ",(0,n.jsx)(t.strong,{children:"contributing"})," Signers, which means only those who have successfully sent individual signatures."]}),"\n",(0,n.jsxs)(t.p,{children:["Note that the already existing ",(0,n.jsx)(t.code,{children:"Certificates"})," will not be updated as this would break the ",(0,n.jsx)(t.code,{children:"Certificate Chain"})," and therefore would involve the bootstraping of a new ",(0,n.jsx)(t.code,{children:"Genesis Certificate"}),"."]}),"\n",(0,n.jsx)(t.p,{children:"This change is transparent to the Signer nodes runned by the SPOs and does not require any specific action from them."}),"\n",(0,n.jsxs)(t.p,{children:["Feel free to reach out to us on the ",(0,n.jsx)(t.a,{href:"https://discord.gg/5kaErDKDRq",children:"Discord channel"})," for questions and/or help."]})]})}function d(e={}){const{wrapper:t}={...(0,r.R)(),...e.components};return t?(0,n.jsx)(t,{...e,children:(0,n.jsx)(l,{...e})}):l(e)}},28453:(e,t,i)=>{i.d(t,{R:()=>o,x:()=>a});var n=i(96540);const r={},s=n.createContext(r);function o(e){const t=n.useContext(s);return n.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function a(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(r):e.components||r:o(e.components),n.createElement(s.Provider,{value:t},e.children)}}}]);