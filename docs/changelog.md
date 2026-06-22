# 鐢瓕顫嗛柨娆掝嚖闁喐鐓￠幍瀣斀

## 娑撯偓閵嗕胶骞嗘晶鍐х瑢瀹搞儱鍙块柧?
### 1.1 pnpm 娑撳秴婀?PATH
**閻滄媽钖?*閿涙瓪pnpm : 閺冪姵纭剁亸?pnpm"妞ょ鐦戦崚顐¤礋 cmdlet`
**閸樼喎娲?*閿涙艾浼愭担婊冨隘 Node.js/pnpm 娑撳秴婀化鑽ょ埠 PATH
**娣囶喖顦?*閿涙碍鐦℃稉顏呮煀缂佸牏顏幍褑顢?```powershell
$env:Path = "C:\Users\hexin\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin;C:\Users\hexin\.cache\codex-runtimes\codex-primary-runtime\dependencies\bin;" + $env:Path
```

### 1.2 npm 娑撳秴婀?PATH閿涘牅绲?pnpm 閸︻煉绱?**閻滄媽钖?*閿涙瓪npm : 閺冪姵纭剁亸?npm"妞ょ鐦戦崚顐¤礋 cmdlet`
**娣囶喖顦?*閿涙氨鏁?`pnpm` 閺囧じ鍞?`npm`閿涘苯鎳℃禒銈呯暚閸忋劌鍚嬬€瑰箍鈧?
### 1.3 Electron/Tauri 閺嬪嫬缂撻懘姘拱鐞氼偄鎷烽悾?**閻滄媽钖?*閿涙瓪[ERR_PNPM_IGNORED_BUILDS] Ignored build scripts`
**娣囶喖顦?*閿涙瓪pnpm approve-builds` 閳?缁岀儤鐗搁崟楣冣偓?閳?閸ョ偠婧呯涵顔款吇 閳?闁插秵鏌?`pnpm install`

### 1.4 Electron 娴滃矁绻橀崚鏈电瑓鏉炶棄銇戠拹?**閻滄媽钖?*閿涙瓪ECONNREFUSED` 鏉╃偞甯?GitHub CDN 鐡掑懏妞?**娣囶喖顦?*閿涙矮濞囬悽銊︾獝鐎规繈鏆呴崓?```powershell
$env:ELECTRON_MIRROR = "https://npmmirror.com/mirrors/electron/"
node node_modules/electron/install.js
```

### 1.5 Rust 瀹搞儱鍙块柧鍓у繁婢?**閻滄媽钖?*閿涙瓪cargo : 閺冪姵纭剁亸?cargo"妞ょ鐦戦崚顐¤礋 cmdlet`
**娣囶喖顦?*閿涙艾鐣ㄧ憗?rustup閿涘牅绔村▎鈩冣偓褝绱?```powershell
Invoke-WebRequest -Uri "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe" -OutFile "$env:TEMP\rustup-init.exe"
& "$env:TEMP\rustup-init.exe" -y
```

### 1.6 MSVC 闁剧偓甯撮崳銊у繁婢?**閻滄媽钖?*閿涙瓪error: linker 'link.exe' not found`
**娣囶喖顦?*閿涙艾鐣ㄧ憗?VS Build Tools閿涘牅绔村▎鈩冣偓褝绱濈痪?3GB閿?```powershell
Invoke-WebRequest -Uri "https://aka.ms/vs/17/release/vs_buildtools.exe" -OutFile "$env:TEMP\vs_buildtools.exe"
& "$env:TEMP\vs_buildtools.exe" --quiet --wait --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended
```
闁插秴鎯庣紒鍫㈩伂閸氬海鏁撻弫鍫涒偓?
---

## 娴滃被鈧焦鏋冩禒鍓佺椽閻?
### 2.1 PowerShell Out-File 閸愭瑥鍙?BOM 婢?**閻滄媽钖?*閿涙SON 閺傚洣娆㈢悮?Tauri/cargo/Vite 閹?`expected value at line 1 column 1` 閹?`Unexpected token '\uFEFF'`
**閸樼喎娲?*閿涙瓪Out-File -Encoding utf8` 娴兼艾婀弬鍥︽婢舵潙鍟撻崗?`EF BB BF`閿涘湐OM閿?**娣囶喖顦?*閿涙氨鏁ら弮?BOM 閻ㄥ嫭鏌熷蹇撳晸閸?```powershell
$utf8 = [System.Text.UTF8Encoding]::new($false)
[System.IO.File]::WriteAllText("path/to/file.json", $content, $utf8)
```

### 2.2 娑擃厽鏋冪捄顖氱窞鐎佃壈鍤?rc.exe 瀹曗晜绨?**閻滄媽钖?*閿涙瓪RC.EXE failed to compile`閿涘矁鐭惧鍕潶閹搭亝鏌囬崷銊よ厬閺傚洤顦?**閸樼喎娲?*閿涙瓙indows Resource Compiler (rc.exe) 娑撳秵鏁幐?Unicode 鐠侯垰绶?**娣囶喖顦?*閿涙岸銆嶉惄顔跨熅瀵板嫪鑵戞稉宥堫洣閸氼偂鑵戦弬鍥х摟缁楋讣绱檂濡楀矂娼版笟璺劮瀵偓閸欐叢 閳?`sticky-notes`閿涘鈧?
---

## 娑撳鈧阜ust / windows-sys 缁鐎烽梽鐑芥Ш

### 3.1 windows-sys 0.59 缁鐎风€规矮绠?| Win32 缁鐎?| windows-sys 鐎圭偤妾猾璇茬€?| 濞夈劍鍓版禍瀣€?|
|-----------|---------------------|---------|
| HWND | `*mut c_void` | 閹稿洭鎷￠敍灞肩瑝閺?isize閵嗕繖!p.is_null()` 閸掋倗鈹?|
| WPARAM | `usize` | 鐟佸憡鏆ｉ弫甯礉閺?`.0` 鐎涙顔?|
| LPARAM | `isize` | 鐟佸憡鏆ｉ弫甯礉閺?`.0` 鐎涙顔?|
| LRESULT | `isize` | 鐟佸憡鏆ｉ弫甯礉娑撳秷鍏橀崘?`LRESULT(0)` |
| SET_WINDOW_POS_FLAGS | `u32` | 缁鐎烽崚顐㈡倳閿涘奔绗夐懗钘夌秼閺嬪嫰鈧姴鍤遍弫鎵暏 |
| SHOW_WINDOW_CMD | `i32` | 娑撳秵妲?u32 |
| FindWindowW 鏉╂柨娲栭崐?| `*mut c_void` | 娑?0 濮ｆ棁绶?閳?閻?`!p.is_null()` |

### 3.2 `*mut c_void` 娑撳秵寮х搾?Send trait
**閻滄媽钖?*閿涙瓪*mut c_void cannot be sent between threads safely`
**娣囶喖顦?*閿涙氨鏁?`isize` 鐎涙ê鍋嶉崷鏉挎絻閸婅壈娉曠痪璺ㄢ柤娴肩娀鈧?```rust
let h = hwnd as isize;  // 娣囨繂鐡?// ... 鐠恒劎鍤庣粙?...
ShowWindow(h as *mut c_void, ...);  // 閹垹顦?```

### 3.3 Inner attribute 韫囧懘銆忛崷銊︽瀮娴犲墎顑囨稉鈧悰?**閻滄媽钖?*閿涙瓪an inner attribute is not permitted in this context`
**娣囶喖顦?*閿涙瓪#![...]` 瑜般垹绱￠惃鍕潣閹冪箑妞よ婀?`use` 鐠囶厼褰炴稊瀣閵嗕焦鏋冩禒鑸垫付妞ゅ爼鍎?```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[cfg(test)]
mod sync;

use std::...;
```

### 3.4 String vs &str 濞村鐦В鏃囩窛
**閻滄媽钖?*閿涙瓪can't compare 'String' with '&str'`
**娣囶喖顦?*閿涙碍绁寸拠鏇氳厬鐠嬪啰鏁?`.as_str()`
```rust
assert!(m.as_str() > "a0");
assert!(orders[1].as_str() < "z0");
```

---

## 閸ユ稏鈧焦鐏﹂弸鍕缁?
### 4.1 娑撱儳顩?SetWindowLongPtrW 鐟曞棛娲?GWLP_WNDPROC
**閸樼喎娲?*閿涙艾鍨忛弬?Tauri 鐎涙劗琚柧?閳?閹锋牗瀚?DPI/DWM 閸氬牊鍨氬畷鈺傜皾
**濮濓絿鈥?*閿涙矮濞囬悽?`SetWindowSubclass` + `DefSubclassProc`

### 4.2 娑撱儳顩﹂崜宥囶伂娴ｈ法鏁?data-tauri-drag-region
**閸樼喎娲?*閿涙矮绗?WM_NCHITTEST 閻?HTCAPTION 鐠侯垳鏁遍崣鎴犳晸缁崵绮虹痪褎濮犻崡?閳?缁ɑ澧?閸楋繝銆?**濮濓絿鈥?*閿涙碍澧滈弻鍕隘 HTML 娣囨繃瀵旂痪顖氬櫍閿涘本瀚嬮幏钘夊弿閺夊啩姘?Win32 鐏?
### 4.3 娑撱儳顩︾粋鑽ゅ殠闁插秴閽╃悰?**閸樼喎娲?*閿涙氨顬囩痪鍧楀櫢閹烘帊绱扮€佃壈鍤ф径褔鍣?updatedAt 閸欐ɑ娲?閳?閸氬本顒為弮鑸靛闁插繐鍟跨粣?**濮濓絿鈥?*閿涙矮绗侀柌宥夋，缁備緤绱皁rder>32鐎涙濡?+ Dirty=0 + 閸掓艾鐣幋?GET 閸氬本顒?---

## 浜斻€佷氦浜掔害瀹?
### 5.1 鎵€鏈夋搷浣滄寚浠ゅ繀椤诲寘鍚畬鏁?PATH 璁剧疆
**鍘熷洜**锛氭柊缁堢 PATH 涓㈠け
**鏍煎紡**锛氭瘡涓渶瑕佺敤鎴锋墽琛岀殑鍛戒护鍧楋紝寮€澶村浐瀹氬甫涓婏細
```powershell
$env:Path = "C:\Users\hexin\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin;C:\Users\hexin\.cache\codex-runtimes\codex-primary-runtime\dependencies\bin;" + $env:Path
```
涔嬪悗绱ц窡瀹為檯鍛戒护銆傜敤鎴峰彲鐩存帴鍏ㄩ€夊鍒剁矘璐淬€
### 4.4 Rust `mod` is a keyword — cannot be used as module path
**Error**: `expected identifier, found keyword 'mod'` when calling `win32::mod::fn()`
**Cause**: `mod.rs` defines the parent module directly. Functions in it are at `win32::fn()`, not `win32::mod::fn()`.
**Fix**: Use `win32::apply_styles(h)` directly, or rename the file.