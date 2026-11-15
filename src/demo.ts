import type { Ledger, Transaction, Category } from './domain';
export function demoLedger(): Ledger {
  const now = new Date(); const current = `${now.getFullYear()}-${String(now.getMonth()+1).padStart(2,'0')}`;
  const rows: [string, number, Category, string, string][] = [
    ['REWE',-6842,'Groceries','Weekly groceries','n26'],['Studio North',385000,'Income','November salary','n26'],['Spotify',-1099,'Subscriptions','Premium Individual','n26'],['Five Elephant',-1250,'Food & drink','Coffee & something sweet','n26'],['Deutsche Bahn',-4900,'Transport','Deutschlandticket','n26'],['ARKET',-8990,'Shopping','A few autumn essentials','n26'],['Vattenfall',-7800,'Home','Monthly electricity','n26'],['EDEKA',-4321,'Groceries','Fresh things','n26'],['Netflix',-1399,'Subscriptions','Standard subscription','n26'],['Mamecha',-2840,'Food & drink','Lunch with friends','n26'],['dm-drogerie markt',-2465,'Health','Everyday essentials','n26'],['Rent · Lindenstraße',-115000,'Home','Monthly rent','n26'],['Freelance project',64000,'Income','Brand design','n26'],['Lidl',-5270,'Groceries','Pantry restock','n26'],['BVG',-380,'Transport','Single ticket','n26'],['Apple',-299,'Subscriptions','iCloud+','n26'],['Savings interest',1860,'Income','Interest payment','ing']
  ];
  const day=Math.min(now.getDate(),28);
  const transactions:Transaction[]=rows.map((r,i)=>({id:`demo-${i}`,merchant:r[0],amount:r[1],category:r[2],note:r[3],accountId:r[4],date:`${current}-${String(Math.max(1,day-Math.floor(i/2))).padStart(2,'0')}`}));
  // Previous months make the cash-flow view useful without manufacturing derived totals.
  for(let month=1;month<=5;month++) {
    const date=new Date(now.getFullYear(),now.getMonth()-month,5).toLocaleDateString('en-CA');
    transactions.push({id:`salary-${month}`,accountId:'n26',merchant:'Studio North',amount:385000,category:'Income',note:'Monthly salary',date},{id:`living-${month}`,accountId:'n26',merchant:'Monthly living expenses',amount:-(222000+month*8500),category:'Home',note:'Demo historical summary',date});
  }
  return {version:1,accounts:[{id:'n26',name:'Everyday',bank:'N26',iban:'DE89 3704 0044 0532 0130 00',balance:428650,color:'mint',kind:'Current account'},{id:'ing',name:'Rainy day',bank:'ING',iban:'DE44 5001 0517 5407 3249 31',balance:1250000,color:'peach',kind:'Savings'},{id:'cash',name:'Pocket money',bank:'Cash',iban:'',balance:8500,color:'lilac',kind:'Cash'}],transactions,budgets:[{id:'b1',category:'Groceries',limit:40000},{id:'b2',category:'Food & drink',limit:20000},{id:'b3',category:'Shopping',limit:15000},{id:'b4',category:'Transport',limit:10000}],drafts:[]};
}
