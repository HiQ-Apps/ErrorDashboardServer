import{g as u}from"./index-DElEkuQ_.js";function f(e,o){for(var n=0;n<o.length;n++){const r=o[n];if(typeof r!="string"&&!Array.isArray(r)){for(const t in r)if(t!=="default"&&!(t in e)){const s=Object.getOwnPropertyDescriptor(r,t);s&&Object.defineProperty(e,t,s.get?s:{enumerable:!0,get:()=>r[t]})}}}return Object.freeze(Object.defineProperty(e,Symbol.toStringTag,{value:"Module"}))}var p,a;function c(){if(a)return p;a=1,p=e,e.displayName="properties",e.aliases=[];function e(o){o.languages.properties={comment:/^[ \t]*[#!].*$/m,"attr-value":{pattern:/(^[ \t]*(?:\\(?:\r\n|[\s\S])|[^\\\s:=])+(?: *[=:] *(?! )| ))(?:\\(?:\r\n|[\s\S])|[^\\\r\n])+/m,lookbehind:!0},"attr-name":/^[ \t]*(?:\\(?:\r\n|[\s\S])|[^\\\s:=])+(?= *[=:]| )/m,punctuation:/[=:]/}}return p}var i=c();const l=u(i),g=f({__proto__:null,default:l},[i]);export{g as p};
