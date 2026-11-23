# slotPlus Core's wallpapers, one for each shelf: card/Core/System/wallpapers/<shelf>.png.
# Run: python3 tools/core_wallpapers/make.py card/Core/System/wallpapers
import numpy as np, math, random, sys
from PIL import Image, ImageFilter, ImageDraw
W,H=720,480
ICONS={
'heart':["..##.##..",".#######.","#########","#########",".#######.","..#####..","...###...","....#...."],
'star':["....#....","....#....","...###...","#########",".#######.","..#####..","..##.##..",".##...##."],
'dpad':["...###...","...###...","...###...","#########","#########","#########","...###...","...###...","...###..."],
'note':["....####","....#..#","....#..#","....#..#","..###.##","######..",".####..."],
'bolt':[".....###","....###.","...###..","..######",".....##.","....##..","...##...","..#....."],
'daisy':["..#.#..",".##.##.","##.o.##","..ooo..","##.o.##",".##.##.","..#.#.."],
'buttons':["...........###",".........#####",".........#####","..###....#####",".#####....###.",".#####........",".#####........","..###........."],
'moon':["..####..",".###....","###.....","###.....","###.....","###.....",".###....","..####.."],
'diamond':["....#....","...###...","..#####..",".#######.","#########",".#######.","..#####..","...###...","....#...."],
'cross':["..#..","..#..","#####","..#..","..#.."],
}
def icon_img(name, col, px, inner=None):
    rows=ICONS[name]; h=len(rows); w=max(len(r) for r in rows)
    pad=1
    im=Image.new('RGBA',((w+2*pad)*px,(h+2*pad)*px),(0,0,0,0))
    a=np.zeros((h+2*pad,w+2*pad),bool)
    for y,r in enumerate(rows):
        for x,c in enumerate(r):
            if c in '#o': a[y+pad,x+pad]=True
    # sticker border: dilate by 1 cell twice
    b=a.copy()
    for _ in range(1):
        n=b.copy()
        n[1:]|=b[:-1]; n[:-1]|=b[1:]; n[:,1:]|=b[:,:-1]; n[:,:-1]|=b[:,1:]
        b=n
    d=ImageDraw.Draw(im)
    for y in range(a.shape[0]):
        for x in range(a.shape[1]):
            if b[y,x]: d.rectangle([x*px,y*px,(x+1)*px-1,(y+1)*px-1],fill=(238,232,220,255))
    for y in range(a.shape[0]):
        for x in range(a.shape[1]):
            if a[y,x]:
                c=col
                ry,rx=y-pad,x-pad
                if 0<=ry<h and 0<=rx<len(rows[ry]) and rows[ry][rx]=='o': c=(240,196,40)
                # simple shading: lighter top-left cell
                if y>0 and not a[y-1,x]: c=tuple(min(255,int(v*1.25)) for v in col)
                d.rectangle([x*px,y*px,(x+1)*px-1,(y+1)*px-1],fill=c+(255,))
    return im
def base(top,bot,seed,weave=False,grain=6):
    rng=np.random.default_rng(seed)
    y=np.linspace(0,1,H)[:,None]; x=np.linspace(-1,1,W)[None,:]
    t=np.array(top,float); b=np.array(bot,float)
    img=t*(1-y[...,None])+b*y[...,None]
    img=np.broadcast_to(img,(H,W,3)).copy()
    # vignette
    r=np.sqrt((x*0.9)**2+((np.linspace(-1,1,H)[:,None])*1.1)**2)
    img*= (1-0.35*np.clip(r-0.3,0,1))[...,None]
    # fabric grain
    n=rng.normal(0,grain,(H,W,1)); img+=n
    if weave:
        xx,yy=np.meshgrid(np.arange(W),np.arange(H))
        img+= (((xx+yy)//3)%2*6-3)[...,None]
    return Image.fromarray(np.clip(img,0,255).astype('uint8'),'RGB').convert('RGBA')
def stick(bg,name,col,px,xy,ang):
    ic=icon_img(name,col,px).rotate(ang,resample=Image.BICUBIC,expand=True)
    sh=Image.new('RGBA',ic.size,(0,0,0,0)); sh.putalpha(ic.getchannel('A').point(lambda v: 90 if v else 0))
    sh=sh.filter(ImageFilter.GaussianBlur(3))
    bg.alpha_composite(sh,(xy[0]+4,xy[1]+5)); bg.alpha_composite(ic,xy)
S={ # key: (top,bottom, stickers[(icon,colour,px,(x,y),angle)], weave)
 'default':((70,48,38),(40,27,22),[('heart',(214,52,62),6,(96,70),-8),('star',(236,190,60),5,(570,64),10)],False),
 'gba':((58,52,112),(30,26,60),[('buttons',(150,120,220),5,(80,60),-6),('star',(236,190,60),5,(590,80),12)],False),
 'gb':((76,84,62),(40,46,32),[('dpad',(60,60,64),6,(84,58),-10),('heart',(150,40,90),5,(586,70),8)],False),
 'gbc':((36,96,96),(18,48,50),[('diamond',(232,200,60),5,(80,62),-8),('heart',(222,80,110),5,(590,70),10)],False),
 'ws':((68,68,82),(34,34,42),[('moon',(236,214,120),6,(92,58),-6),('star',(170,200,236),5,(588,74),12)],False),
 'lynx':((72,56,46),(36,28,24),[('bolt',(240,140,40),6,(90,54),-8),('cross',(236,120,40),7,(600,74),14)],False),
 'pokemini':((42,64,120),(22,32,64),[('star',(236,214,90),5,(86,64),-12),('note',(250,226,120),5,(592,70),8)],False),
 'gg':((30,36,58),(14,16,28),[('bolt',(60,170,230),6,(86,56),-6),('buttons',(220,70,70),5,(570,70),8)],False),
 'ngp':((48,48,54),(24,24,28),[('diamond',(230,200,70),5,(84,60),-10),('cross',(200,60,50),7,(600,70),12)],True),
 'tg16':((84,70,58),(42,34,28),[('star',(240,130,40),5,(86,64),-8),('buttons',(220,80,60),5,(570,70),8)],False),
 'nds':((64,72,88),(32,36,46),[('dpad',(70,74,86),5,(90,58),-6),('star',(150,190,236),5,(596,74),10)],False),
 'pokedaisy':((52,96,50),(26,48,24),[('daisy',(250,250,244),6,(82,56),-10),('daisy',(250,250,244),5,(596,74),12)],False),
 'favorites':((100,38,56),(52,18,28),[('heart',(232,62,80),6,(88,60),-10),('heart',(250,160,180),5,(596,70),12)],False),
 'recents':((54,58,70),(28,30,36),[('note',(160,200,240),5,(86,60),-8),('star',(236,190,60),5,(596,74),10)],False),
}
out=sys.argv[1]
import os; os.makedirs(out,exist_ok=True)
for i,(k,(top,bot,stk,weave)) in enumerate(S.items()):
    bg=base(top,bot,i,weave)
    # a faint pattern of the shelf's first motif across the mat
    n0,c0=stk[0][0],stk[0][1]
    tile=icon_img(n0,c0,3)
    a=tile.getchannel('A').point(lambda v: int(v*0.07))
    tile.putalpha(a)
    for yy in range(-20,H,64):
        for xx in range(-20+((yy//64)%2)*48,W,96):
            bg.alpha_composite(tile,(xx,yy))
    for (n,c,px,xy,ang) in stk: stick(bg,n,c,int(px*1.7),(xy[0]-20,xy[1]-24),ang)
    # daisy centres for pokedaisy
    bg.convert('RGB').quantize(colors=256,method=Image.MEDIANCUT,dither=Image.FLOYDSTEINBERG).save(f'{out}/{k}.png',optimize=True)
