-- cc.lua: компилятор подмножества C в ассемблер SummumOS. Написан на Lua, потому что почему бы и нет.
-- Вход: глобальная строка C_SOURCE. Выход: текст ассемблера (return).
-- Умеет: int, char, void, указатели, массивы, функции (до сколькоугодно параметров), if/else,
-- while, for, break, continue, return, все обычные операторы, строки, символы, ++ --, += -= и т.д.
-- Не умеет: struct, enum, float, switch, указатели на функции, typedef. int без знака в / и %.

-- чертёж №32: две вилки спорят, кто из них компилятор.
local PRELUDE = [[
void print(char *s) { while (*s) { putchar(*s); s = s + 1; } }
void putnl() { putchar(10); }
void print_num(int n) {
    if (n < 0) { putchar(45); n = -n; }
    if (n >= 10) print_num(n / 10);
    putchar(48 + n % 10);
}
]]

local KW = {}
for w in ("int char void short long unsigned signed const static if else while for return break continue sizeof"):gmatch("%a+") do KW[w] = true end

local TWO = { "==", "!=", "<=", ">=", "&&", "||", "++", "--", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<", ">>" }
local THREE = { "<<=", ">>=" }

local function unesc(s, line)
    local out, i = {}, 1
    while i <= #s do
        local c = s:sub(i, i)
        if c == "\\" then
            i = i + 1
            local e = s:sub(i, i)
            if e == "n" then out[#out + 1] = 10
            elseif e == "t" then out[#out + 1] = 9
            elseif e == "r" then out[#out + 1] = 13
            elseif e == "0" then out[#out + 1] = 0
            elseif e == "\\" then out[#out + 1] = 92
            elseif e == '"' then out[#out + 1] = 34
            elseif e == "'" then out[#out + 1] = 39
            else error("строка " .. line .. ": неизвестный escape \\" .. e, 0) end
        else
            out[#out + 1] = c:byte()
        end
        i = i + 1
    end
    return out
end

local function lex(src)
    local toks, i, line, n = {}, 1, 1, #src
    local function add(t, v) toks[#toks + 1] = { t = t, v = v, line = line } end
    while i <= n do
        local c = src:sub(i, i)
        if c == "\n" then
            line = line + 1
            i = i + 1
        elseif c:find("^%s") then
            i = i + 1
        elseif src:sub(i, i + 1) == "//" then
            i = src:find("\n", i, true) or (n + 1)
        elseif src:sub(i, i + 1) == "/*" then
            local s, e = src:find("*/", i + 2, true)
            if not s then error("строка " .. line .. ": не закрыт комментарий", 0) end
            for _ in src:sub(i, e):gmatch("\n") do line = line + 1 end
            i = e + 1
        elseif c == "#" then
            i = src:find("\n", i, true) or (n + 1)
        elseif c:find("^[%a_]") then
            local s, e = src:find("^[%w_]+", i)
            local w = src:sub(s, e)
            add(KW[w] and "kw" or "id", w)
            i = e + 1
        elseif c:find("^%d") then
            local s, e = src:find("^0[xX]%x+", i)
            if s then
                add("num", tonumber(src:sub(s, e)))
            else
                s, e = src:find("^%d+", i)
                add("num", tonumber(src:sub(s, e)))
            end
            i = e + 1
        elseif c == '"' then
            local j = i + 1
            while j <= n and src:sub(j, j) ~= '"' do
                if src:sub(j, j) == "\\" then j = j + 1 end
                j = j + 1
            end
            add("str", unesc(src:sub(i + 1, j - 1), line))
            i = j + 1
        elseif c == "'" then
            local j = i + 1
            while j <= n and src:sub(j, j) ~= "'" do
                if src:sub(j, j) == "\\" then j = j + 1 end
                j = j + 1
            end
            add("num", unesc(src:sub(i + 1, j - 1), line)[1])
            i = j + 1
        else
            local found
            for _, op in ipairs(THREE) do
                if src:sub(i, i + 2) == op then found = op end
            end
            if not found then
                for _, op in ipairs(TWO) do
                    if src:sub(i, i + 1) == op then found = op end
                end
            end
            found = found or c
            add("op", found)
            i = i + #found
        end
    end
    add("eof", "<eof>")
    return toks
end

local function T(b, p, arr) return { b = b, p = p or 0, arr = arr } end

-- ---- peephole: чистим за собой. Наш компилятор честно кладёт всё в стек, а мы потом
-- ---- делаем вид, что так и было задумано (и код становится вдвое короче и быстрее).
local function pl_parse(s)
    if s:sub(1, 4) ~= "    " then return { label = s:sub(1, -2) } end
    local op, rest = s:match("^    (%S+) ?(.*)$")
    local args = {}
    for a in rest:gmatch("[^, ]+") do args[#args + 1] = a end
    return { op = op, args = args }
end

local function pl_fmt(i)
    if i.label then return i.label .. ":" end
    local s = "    " .. i.op
    if #i.args > 0 then s = s .. " " .. table.concat(i.args, ", ") end
    return s
end

local function pl_is(i, op, a1, a2, a3)
    if not i or i.label or i.op ~= op then return false end
    if a1 and i.args[1] ~= a1 then return false end
    if a2 and i.args[2] ~= a2 then return false end
    if a3 and i.args[3] ~= a3 then return false end
    return true
end

-- [reg+off] -> reg, off
local function pl_mem(a)
    if not a then return nil end
    return a:match("^%[(%w+)%+(.*)%]$")
end

local PL_JUMPS = { jmp = true, jz = true, jnz = true, je = true, jne = true, jlt = true, jge = true }
local PL_INV = { jz = "jnz", jnz = "jz", je = "jne", jne = "je", jlt = "jge", jge = "jlt" }

local function pl_isjump(i) return i and not i.label and PL_JUMPS[i.op] end
local function pl_target(i) return i.args[#i.args] end

local function pl_refs(L)
    local r = {}
    for _, i in ipairs(L) do
        if pl_isjump(i) then
            local t = pl_target(i)
            r[t] = (r[t] or 0) + 1
        end
    end
    return r
end

local function pl_ren(a, to)
    if a == "r0" then return to end
    if a:sub(1, 4) == "[r0+" then return "[" .. to .. "+" .. a:sub(5) end
    return a
end
local function pl_ren01(a) return pl_ren(a, "r1") end

local function pl_renins(s, to)
    local na = {}
    for q, x in ipairs(s.args) do na[q] = pl_ren(x, to) end
    return { op = s.op, args = na }
end

-- команда считает r0 с нуля, не глядя на старое r0
local function pl_indep(i)
    if i.label then return false end
    if i.op == "ldi" or (i.op == "addi" and i.args[2] == "r6") then return true end
    if (i.op == "ld" or i.op == "ldb") then
        local r = pl_mem(i.args[2])
        return r == "r6" or r == "r5"
    end
    return false
end

-- между парой push/pop: ищем парный pop (на том же уровне стека), возвращаем индекс и "чистоту" (нет записей и вызовов)
local function pl_match(L, from)
    local depth, j, pure = 1, from, true
    while L[j] do
        local x = L[j]
        if not x.label then
            if x.op == "push" then depth = depth + 1
            elseif x.op == "pop" then depth = depth - 1
            elseif x.op == "addi" and x.args[1] == "sp" and x.args[2] == "sp" then
                depth = depth - math.floor(tonumber(x.args[3]) / 2)
            elseif x.op == "call" or x.op == "st" or x.op == "stb" then pure = false end
            if depth == 0 then return j, pure end
        end
        j = j + 1
    end
    return nil, pure
end

-- команда "простая": трогает только r0 и r6, без стека, вызовов и переходов
local function pl_simple(i)
    if i.label then return false end
    local o = i.op
    if o ~= "ldi" and o ~= "ld" and o ~= "ldb" and o ~= "addi" then return false end
    for k, a in ipairs(i.args) do
        if k == 1 and a ~= "r0" then return false end
        if k > 1 then
            local r = pl_mem(a)
            if r then
                if r ~= "r0" and r ~= "r6" and r ~= "r5" then return false end
            elseif a:match("^r%d$") or a == "sp" then
                if a ~= "r0" and a ~= "r6" then return false end
            end
        end
    end
    return true
end

local function pl_pass1(L)
    local O, ch, i = {}, false, 1
    local n = #L
    while i <= n do
        local a, b = L[i], L[i + 1]
        local o = a.op
        -- addi r0,r6,N ; ld r0,[r0+0]  ->  ld r0,[r6+N]
        if o ~= "addi" and o ~= "ldi" and o ~= "push" then
            O[#O + 1] = a; i = i + 1
        elseif pl_is(a, "addi") and a.args[2] == "r6" and b and not b.label and (b.op == "ld" or b.op == "ldb")
            and b.args[1] == a.args[1] and b.args[2] == "[" .. a.args[1] .. "+0]" then
            O[#O + 1] = { op = b.op, args = { a.args[1], "[r6+" .. a.args[3] .. "]" } }
            i = i + 2; ch = true
        -- ldi r0,A ; ldi r1,B ; add/sub/mul r0,r0,r1  ->  ldi r0,A op B
        elseif pl_is(a, "ldi", "r0") and pl_is(b, "ldi", "r1") and tonumber(a.args[2]) and tonumber(b.args[2])
            and L[i + 2] and (pl_is(L[i + 2], "add", "r0", "r0", "r1") or pl_is(L[i + 2], "sub", "r0", "r0", "r1") or pl_is(L[i + 2], "mul", "r0", "r0", "r1")) then
            local x, y, o = tonumber(a.args[2]), tonumber(b.args[2]), L[i + 2].op
            local v = (o == "add") and (x + y) or ((o == "sub") and (x - y) or (x * y))
            O[#O + 1] = { op = "ldi", args = { "r0", tostring(math.floor(v % 65536)) } }
            i = i + 3; ch = true
        -- ldi r0,G_x ; ld r0,[r0+0]  ->  ld r0,[r5+G_x]   (r5 всегда ноль)
        elseif pl_is(a, "ldi") and a.args[2]:sub(1, 2) == "G_" and b and not b.label
            and (b.op == "ld" or b.op == "ldb") and b.args[1] == a.args[1] and b.args[2] == "[" .. a.args[1] .. "+0]" then
            O[#O + 1] = { op = b.op, args = { a.args[1], "[r5+" .. a.args[2] .. "]" } }
            i = i + 2; ch = true
        -- ldi r0,G_x ; mov r1,r0 ; ld r0,[r1+0] ; addi r2,r0,K ; st [r1+0],r2
        elseif pl_is(a, "ldi", "r0") and a.args[2]:sub(1, 2) == "G_" and pl_is(b, "mov", "r1", "r0")
            and L[i + 2] and (pl_is(L[i + 2], "ld", "r0", "[r1+0]") or pl_is(L[i + 2], "ldb", "r0", "[r1+0]"))
            and pl_is(L[i + 3], "addi", "r2", "r0")
            and (pl_is(L[i + 4], "st", "[r1+0]", "r2") or pl_is(L[i + 4], "stb", "[r1+0]", "r2")) then
            local ld, st = L[i + 2], L[i + 4]
            O[#O + 1] = { op = ld.op, args = { "r0", "[r5+" .. a.args[2] .. "]" } }
            O[#O + 1] = L[i + 3]
            O[#O + 1] = { op = st.op, args = { "[r5+" .. a.args[2] .. "]", "r2" } }
            i = i + 5; ch = true
        -- push r0 ; <простое> ; mov r1,r0 ; pop r0  ->  <простое в r1>
        elseif pl_is(a, "push", "r0") then
            local j = i + 1
            while L[j] and pl_simple(L[j]) do j = j + 1 end
            if j > i + 1 and pl_is(L[j], "mov", "r1", "r0") and pl_is(L[j + 1], "pop", "r0") then
                for k = i + 1, j - 1 do
                    local s = L[k]
                    local na = {}
                    for q, x in ipairs(s.args) do na[q] = pl_ren01(x) end
                    O[#O + 1] = { op = s.op, args = na }
                end
                i = j + 2; ch = true
            else
                O[#O + 1] = a; i = i + 1
            end
        -- addi r0,r6,N ; mov r1,r0 ; ld r0,[r1+0] ; addi r2,r0,K ; st [r1+0],r2  (это i++)
        elseif pl_is(a, "addi", "r0", "r6") and pl_is(b, "mov", "r1", "r0")
            and L[i + 2] and (pl_is(L[i + 2], "ld", "r0", "[r1+0]") or pl_is(L[i + 2], "ldb", "r0", "[r1+0]"))
            and pl_is(L[i + 3], "addi", "r2", "r0")
            and (pl_is(L[i + 4], "st", "[r1+0]", "r2") or pl_is(L[i + 4], "stb", "[r1+0]", "r2")) then
            local ld, st = L[i + 2], L[i + 4]
            O[#O + 1] = { op = ld.op, args = { "r0", "[r6+" .. a.args[3] .. "]" } }
            O[#O + 1] = L[i + 3]
            O[#O + 1] = { op = st.op, args = { "[r6+" .. a.args[3] .. "]", "r2" } }
            i = i + 5; ch = true
        -- ldi r1,X ; add r0,r1,r0 ; ld/ldb r0,[r0+0]  ->  ld/ldb r0,[r0+X]
        elseif pl_is(a, "ldi", "r1") and pl_is(b, "add", "r0", "r1", "r0") and L[i + 2] and not L[i + 2].label
            and (L[i + 2].op == "ld" or L[i + 2].op == "ldb") and L[i + 2].args[1] == "r0" and L[i + 2].args[2] == "[r0+0]" then
            O[#O + 1] = { op = L[i + 2].op, args = { "r0", "[r0+" .. a.args[2] .. "]" } }
            i = i + 3; ch = true
        -- addi r1,r6,N ; add r0,r1,r0 ; ld/ldb r0,[r0+0]  ->  add r0,r0,r6 ; ld/ldb r0,[r0+N]
        elseif pl_is(a, "addi", "r1", "r6") and pl_is(b, "add", "r0", "r1", "r0") and L[i + 2] and not L[i + 2].label
            and (L[i + 2].op == "ld" or L[i + 2].op == "ldb") and L[i + 2].args[1] == "r0" and L[i + 2].args[2] == "[r0+0]" then
            O[#O + 1] = { op = "add", args = { "r0", "r0", "r6" } }
            O[#O + 1] = { op = L[i + 2].op, args = { "r0", "[r0+" .. a.args[3] .. "]" } }
            i = i + 3; ch = true
        -- ldi r1,K ; cmp r0,r1  ->  cmpi r0,K
        elseif pl_is(a, "ldi", "r1") and pl_is(b, "cmp", "r0", "r1") and a.args[2]:match("^%d+$") then
            O[#O + 1] = { op = "cmpi", args = { "r0", a.args[2] } }
            i = i + 2; ch = true
        else
            O[#O + 1] = a; i = i + 1
        end
    end
    return O, ch
end

-- адрес закинули в стек, посчитали значение, достали: переставляем местами, стек не нужен
local function pl_pass2(L)
    local O, ch, i = {}, false, 1
    local n = #L
    while i <= n do
        local p, q = L[i], L[i + 1]
        local done = false
        local kind = nil
        if p and not p.label and q and q.op == "push" then
            if pl_is(p, "addi", "r0", "r6") or pl_is(p, "ldi", "r0") then kind = "addr"
            elseif pl_is(p, "ld", "r0") and (p.args[2] or ""):match("^%[r6%+") then kind = "var" end
        end
        if kind and pl_is(q, "push", "r0") then
            local base = nil
            if pl_is(p, "addi", "r0", "r6") then base = "[r6+" .. p.args[3] .. "]"
            elseif pl_is(p, "ldi", "r0") and p.args[2]:sub(1, 2) == "G_" then base = "[r5+" .. p.args[2] .. "]" end
            local j, pure = pl_match(L, i + 2)
            -- x op= e :  push r0; S; mov r2,r0; pop r1; push r1; LD r0,[r1+0]; mov r1,r2; OP...; pop r1; ST [r1+0],r0
            if base and j and pl_is(L[j], "pop", "r1") and pl_is(L[j - 1], "mov", "r2", "r0") and pl_is(L[j + 1], "push", "r1")
                and L[j + 2] and (pl_is(L[j + 2], "ld", "r0", "[r1+0]") or pl_is(L[j + 2], "ldb", "r0", "[r1+0]"))
                and pl_is(L[j + 3], "mov", "r1", "r2") then
                local k = j + 4
                while L[k] and not pl_is(L[k], "pop", "r1") do k = k + 1 end
                local st = L[k + 1]
                if L[k] and st and not st.label and (st.op == "st" or st.op == "stb") and st.args[1] == "[r1+0]" and st.args[2] == "r0" then
                    for z = i + 2, j - 2 do O[#O + 1] = L[z] end
                    O[#O + 1] = L[j - 1]
                    O[#O + 1] = { op = L[j + 2].op, args = { "r0", base } }
                    O[#O + 1] = L[j + 3]
                    for z = j + 4, k - 1 do O[#O + 1] = L[z] end
                    O[#O + 1] = { op = st.op, args = { base, "r0" } }
                    i = k + 2; ch = true; done = true
                end
            end
            if not done then
                local c = L[(j or 0) + 1]
                if j and pl_is(L[j], "pop", "r1") and (kind ~= "var" or pure) and c and not c.label
                    and (((c.op == "st" or c.op == "stb") and c.args[1] == "[r1+0]" and c.args[2] == "r0")
                        or pl_is(c, "add", "r0", "r1", "r0")) then
                    for z = i + 2, j - 1 do O[#O + 1] = L[z] end
                    if (c.op == "st" or c.op == "stb") and pl_is(p, "addi", "r0", "r6") then
                        O[#O + 1] = { op = c.op, args = { "[r6+" .. p.args[3] .. "]", "r0" } }
                    elseif (c.op == "st" or c.op == "stb") and base then
                        O[#O + 1] = { op = c.op, args = { base, "r0" } }
                    else
                        O[#O + 1] = pl_renins(p, "r1")
                        O[#O + 1] = c
                    end
                    i = j + 2; ch = true; done = true
                end
            end
        end
        if not done then O[#O + 1] = L[i]; i = i + 1 end
    end
    return O, ch
end

-- a OP (сложное): левое простое -- считаем его после правого, push/pop не нужны
local PL_ARITH = { add = true, sub = true, mul = true, div = true, mod = true, ["and"] = true, ["or"] = true, xor = true }
local function pl_pass6(L)
    local O, ch, i = {}, false, 1
    local n = #L
    while i <= n do
        local a = L[i]
        local done = false
        if a.op == "push" then
            -- левая часть: самая последняя цепочка простых команд, что уже лежит в O
            local k = #O
            local ok = false
            while k >= 1 and pl_simple(O[k]) do
                if pl_indep(O[k]) then ok = true; break end
                k = k - 1
            end
            if ok then
                local j, pure = pl_match(L, i + 1)
                if j and pl_is(L[j], "pop", "r0") and pl_is(L[j - 1], "mov", "r1", "r0") and L[j + 1] and not L[j + 1].label
                    and PL_ARITH[L[j + 1].op] and L[j + 1].args[1] == "r0" and L[j + 1].args[2] == "r0" and L[j + 1].args[3] == "r1" then
                    local reads = false
                    for z = k, #O do if O[z].op == "ld" or O[z].op == "ldb" then reads = true end end
                    if pure or not reads then
                        local left = {}
                        for z = k, #O do left[#left + 1] = O[z] end
                        for z = #O, k, -1 do O[z] = nil end
                        for z = i + 1, j - 2 do O[#O + 1] = L[z] end
                        for z = 1, #left do O[#O + 1] = pl_renins(left[z], "r1") end
                        O[#O + 1] = { op = L[j + 1].op, args = { "r0", "r1", "r0" } }
                        i = j + 2; ch = true; done = true
                    end
                end
            end
        end
        if not done then O[#O + 1] = a; i = i + 1 end
    end
    return O, ch
end

-- запись в массив: адрес = база + индекс, база уезжает прямо в команду st
local function pl_pass5(L)
    local O, ch, i = {}, false, 1
    local n = #L
    while i <= n do
        local a, b, c = L[i], L[i + 1], L[i + 2]
        local done = false
        local base = nil
        local o = a.op
        if o ~= "ldi" and o ~= "addi" and o ~= "push" then
            -- ничего интересного
        elseif pl_is(a, "ldi", "r1") and pl_is(b, "add", "r0", "r1", "r0") and pl_is(c, "push", "r0") then
            base = a.args[2]
        elseif pl_is(a, "addi", "r1", "r6") and pl_is(b, "add", "r0", "r1", "r0") and pl_is(c, "push", "r0") then
            base = a.args[3]
        end
        if base then
            local j = pl_match(L, i + 3)
            local st = L[(j or 0) + 1]
            if j and pl_is(L[j], "pop", "r1") and st and not st.label and (st.op == "st" or st.op == "stb")
                and st.args[1] == "[r1+0]" and st.args[2] == "r0" then
                -- простое значение: mov r1,r0 ; V (в r2) ; st [r1+base],r2
                local simple = (j > i + 3)
                for z = i + 3, j - 1 do if not pl_simple(L[z]) then simple = false end end
                if pl_is(a, "addi", "r1", "r6") then O[#O + 1] = { op = "add", args = { "r0", "r0", "r6" } } end
                if simple then
                    O[#O + 1] = { op = "mov", args = { "r1", "r0" } }
                    for z = i + 3, j - 1 do O[#O + 1] = pl_renins(L[z], "r2") end
                    O[#O + 1] = { op = st.op, args = { "[r1+" .. base .. "]", "r2" } }
                else
                    O[#O + 1] = c
                    for z = i + 3, j - 1 do O[#O + 1] = L[z] end
                    O[#O + 1] = L[j]
                    O[#O + 1] = { op = st.op, args = { "[r1+" .. base .. "]", "r0" } }
                end
                i = j + 2; ch = true; done = true
            end
        elseif pl_is(a, "push", "r0") then
            local j = pl_match(L, i + 1)
            local st = L[(j or 0) + 1]
            if j and j > i + 1 and pl_is(L[j], "pop", "r1") and st and not st.label and (st.op == "st" or st.op == "stb")
                and st.args[1] == "[r1+0]" and st.args[2] == "r0" then
                local simple = true
                for z = i + 1, j - 1 do if not pl_simple(L[z]) then simple = false end end
                if simple then
                    O[#O + 1] = { op = "mov", args = { "r1", "r0" } }
                    for z = i + 1, j - 1 do O[#O + 1] = pl_renins(L[z], "r2") end
                    O[#O + 1] = { op = st.op, args = { "[r1+0]", "r2" } }
                    i = j + 2; ch = true; done = true
                end
            end
        end
        if not done then O[#O + 1] = a; i = i + 1 end
    end
    return O, ch
end

-- превращение логических значений в прыжки, уборка переходов
local function pl_pass3(L)
    local ch = false
    local refs = pl_refs(L)
    local O, i, n = {}, 1, #L
    local redirect = {}
    while i <= n do
        local a = L[i]
        local o = a.op
        -- ldi r0,A ; jmp Le ; Lf: ; ldi r0,B ; Le: ; jz/jnz r0,X
        if o ~= "ldi" and o ~= "jmp" and not PL_INV[o] then
            O[#O + 1] = a; i = i + 1
        elseif pl_is(a, "ldi", "r0") and a.args[2]:match("^[01]$") and pl_is(L[i + 1], "jmp") and L[i + 2] and L[i + 2].label
            and pl_is(L[i + 3], "ldi", "r0") and L[i + 3].args[2]:match("^[01]$") and L[i + 3].args[2] ~= a.args[2]
            and L[i + 4] and L[i + 4].label == L[i + 1].args[1] and refs[L[i + 4].label] == 1
            and (pl_is(L[i + 5], "jz", "r0") or pl_is(L[i + 5], "jnz", "r0")) then
            local lf, fin = L[i + 2].label, L[i + 5]
            local x = fin.args[2]
            local av = tonumber(a.args[2])
            local taken = (fin.op == "jz" and av == 0) or (fin.op == "jnz" and av ~= 0)
            if taken then
                O[#O + 1] = { op = "jmp", args = { x } }
                O[#O + 1] = L[i + 2]
            else
                redirect[lf] = x
            end
            i = i + 6; ch = true
        -- J L ; jmp X ; L:   ->  inv(J) X ; L:
        elseif pl_isjump(a) and a.op ~= "jmp" and pl_is(L[i + 1], "jmp") and L[i + 2] and L[i + 2].label == pl_target(a) then
            local na = {}
            for k, v in ipairs(a.args) do na[k] = v end
            na[#na] = L[i + 1].args[1]
            O[#O + 1] = { op = PL_INV[a.op], args = na }
            O[#O + 1] = L[i + 2]
            i = i + 3; ch = true
        -- jmp L ; L:
        elseif pl_is(a, "jmp") and L[i + 1] and L[i + 1].label == a.args[1] then
            i = i + 1; ch = true
        else
            O[#O + 1] = a; i = i + 1
        end
    end
    if next(redirect) then
        for _, x in ipairs(O) do
            if pl_isjump(x) then
                local t = pl_target(x)
                local guard = 0
                while redirect[t] and guard < 20 do t = redirect[t]; guard = guard + 1 end
                x.args[#x.args] = t
            end
        end
    end
    -- метки, на которые никто не прыгает
    refs = pl_refs(O)
    local O2 = {}
    for _, x in ipairs(O) do
        if x.label and x.label:match("^L%d+$") and not refs[x.label] then ch = true
        else O2[#O2 + 1] = x end
    end
    return O2, ch
end

-- сравнение с константой: ldi r1,K ; ldi r2,0x8000 ; xor r0 ; xor r1 ; cmp ; jcc  ->  короче
local function pl_pass4(L)
    local O, ch, i = {}, false, 1
    local n = #L
    while i <= n do
        local a = L[i]
        if a.op == "ldi" and pl_is(a, "ldi", "r1") and a.args[2]:match("^%d+$") and pl_is(L[i + 1], "ldi", "r2", "0x8000")
            and pl_is(L[i + 2], "xor", "r0", "r0", "r2") and pl_is(L[i + 3], "xor", "r1", "r1", "r2") then
            local k = tonumber(a.args[2]) % 65536
            local kb = (k + 32768) % 65536
            local c, j = L[i + 4], L[i + 5]
            if c and pl_is(c, "cmp", "r0", "r1") and (pl_is(j, "jlt") or pl_is(j, "jge")) then
                O[#O + 1] = L[i + 1]; O[#O + 1] = L[i + 2]
                O[#O + 1] = { op = "cmpi", args = { "r0", tostring(kb) } }
                O[#O + 1] = j
                i = i + 6; ch = true
            elseif c and pl_is(c, "cmp", "r1", "r0") and (pl_is(j, "jlt") or pl_is(j, "jge")) and kb < 65535 then
                O[#O + 1] = L[i + 1]; O[#O + 1] = L[i + 2]
                O[#O + 1] = { op = "cmpi", args = { "r0", tostring(kb + 1) } }
                O[#O + 1] = { op = (j.op == "jlt") and "jge" or "jlt", args = j.args }
                i = i + 6; ch = true
            else
                O[#O + 1] = a; i = i + 1
            end
        else
            O[#O + 1] = a; i = i + 1
        end
    end
    return O, ch
end

local function optimize_canonical(lines)
    local L = {}
    for _, s in ipairs(lines) do L[#L + 1] = pl_parse(s) end
    for round = 1, 12 do
        local c1, c2, c3, c4, c5, c6
        L, c1 = pl_pass1(L)
        L, c2 = pl_pass2(L)
        L, c6 = pl_pass6(L)
        L, c5 = pl_pass5(L)
        L, c3 = pl_pass3(L)
        L, c4 = pl_pass4(L)
        if not (c1 or c2 or c3 or c4 or c5 or c6) then break end
    end
    local out = {}
    for _, i in ipairs(L) do out[#out + 1] = pl_fmt(i) end
    return out
end

-- компилятор понюхал автобус и стал шестнадцатеричным.
-- байт ест табуретку. табуретка выполняет return.
local skolko_vilok_na_kukhne = SKOLKO_DERUTSYA_VILKI
if skolko_vilok_na_kukhne == nil then skolko_vilok_na_kukhne = 3 end
if type(skolko_vilok_na_kukhne) ~= "number" or skolko_vilok_na_kukhne ~= math.floor(skolko_vilok_na_kukhne)
    or skolko_vilok_na_kukhne < 0 or skolko_vilok_na_kukhne > 8 then
    error("SKOLKO_DERUTSYA_VILKI: ожидалось целое число от 0 до 8", 0)
end
KTO_SYEL_TABURETKU = { rounds = 0, visited = 0, inserted = 0, removed = 0, peak_nodes = 0 }

-- если стало понятно — выключи монитор и пересчитай ноздри.
-- здесь лежит вторник. не будить: он unsigned.
-- инструкция для голубя: клюнуть ноль, расписаться единицей.
local function vilka_nabiraet_musor(L, round)
    local O = {}
    for k, ins in ipairs(L) do
        local name = "__dispute_" .. round .. "_" .. k
        O[#O + 1] = { label = name, dispute = true }
        O[#O + 1] = { op = "mov", args = { "r0", "r0" }, dispute = true }
        O[#O + 1] = pl_parse(pl_fmt(ins))
        KTO_SYEL_TABURETKU.visited = KTO_SYEL_TABURETKU.visited + 1
        KTO_SYEL_TABURETKU.inserted = KTO_SYEL_TABURETKU.inserted + 2
    end
    KTO_SYEL_TABURETKU.peak_nodes = math.max(KTO_SYEL_TABURETKU.peak_nodes, #O)
    return O
end

-- проверено картофелем. картофель отказался от комментариев.
-- это не цикл, это сосиска пошла по кругу за справкой.
local function lozhka_vymetaet_musor(L, round)
    local O, k = {}, 1
    while k <= #L do
        local mark, move, ins = L[k], L[k + 1], L[k + 2]
        local name = "__dispute_" .. round .. "_" .. (#O + 1)
        if not mark.dispute or mark.label ~= name or not move or not move.dispute
            or not pl_is(move, "mov", "r0", "r0") or #move.args ~= 2
            or not ins or ins.dispute then
            error("спор оптимизаторов: повреждён протокол заседания", 0)
        end
        O[#O + 1] = pl_parse(pl_fmt(ins))
        KTO_SYEL_TABURETKU.visited = KTO_SYEL_TABURETKU.visited + 3
        KTO_SYEL_TABURETKU.removed = KTO_SYEL_TABURETKU.removed + 2
        k = k + 3
    end
    return O
end

local function optimize(lines)
    -- память резиновая, но только в четверг и только у соседа.
    local canonical = optimize_canonical(lines)
    if skolko_vilok_na_kukhne == 0 then return canonical end
    local L = {}
    for _, s in ipairs(lines) do L[#L + 1] = pl_parse(s) end
    for round = 1, skolko_vilok_na_kukhne do
        L = vilka_nabiraet_musor(L, round)
        L = lozhka_vymetaet_musor(L, round)
        KTO_SYEL_TABURETKU.rounds = KTO_SYEL_TABURETKU.rounds + 1
    end
    local restored = {}
    for _, ins in ipairs(L) do restored[#restored + 1] = pl_fmt(ins) end
    if table.concat(restored, "\n") ~= table.concat(lines, "\n") then
        error("спор оптимизаторов: исходный IR не восстановлен", 0)
    end
    local out = optimize_canonical(restored)
    if table.concat(out, "\n") ~= table.concat(canonical, "\n") then
        error("спор оптимизаторов: ассемблер отличается от канонического", 0)
    end
    return out
end


local function compile(src)
    local toks = lex(PRELUDE .. "\n" .. src)
    -- строки в прелюдии сдвигают номера, ничего страшного
    local tp = 1
    local cur = {}
    local data = {}
    local strings = {}
    local globals, funcs = {}, { putchar = { ret = T("int") }, getchar = { ret = T("int") } }
    local scopes, fsize, retlabel = {}, 0, nil
    local loops = {}
    local nlabel = 0

    local function cerr(msg)
        error("строка " .. toks[tp].line .. ": " .. msg .. " (около '" .. tostring(toks[tp].v) .. "')", 0)
    end
    local function emit(s) cur[#cur + 1] = "    " .. s end
    local function label(l) cur[#cur + 1] = l .. ":" end
    local function newlabel() nlabel = nlabel + 1; return "L" .. nlabel end

    local function isop(v)
        local t = toks[tp]
        return (t.t == "op" or t.t == "kw") and t.v == v
    end
    local function accept(v)
        if isop(v) then tp = tp + 1; return true end
        return false
    end
    local function expect(v)
        if not accept(v) then cerr("ожидалось '" .. v .. "'") end
    end
    local function expect_id()
        local t = toks[tp]
        if t.t ~= "id" then cerr("ожидалось имя") end
        tp = tp + 1
        return t.v
    end

    local function is_type_start()
        local t = toks[tp]
        return t.t == "kw" and (t.v == "int" or t.v == "char" or t.v == "void" or t.v == "short" or t.v == "long"
            or t.v == "unsigned" or t.v == "signed" or t.v == "const" or t.v == "static")
    end

    local function parse_type()
        local base
        while true do
            local t = toks[tp]
            if t.t == "kw" and (t.v == "const" or t.v == "static" or t.v == "unsigned" or t.v == "signed") then
                tp = tp + 1
            elseif t.t == "kw" and (t.v == "int" or t.v == "short" or t.v == "long") then
                base = "int"; tp = tp + 1
            elseif t.t == "kw" and (t.v == "char" or t.v == "void") then
                base = t.v; tp = tp + 1
            else
                break
            end
        end
        if not base then base = "int" end
        local p = 0
        while accept("*") do p = p + 1 end
        return T(base, p)
    end

    -- размер того, на что указывает указатель (или элемента массива)
    local function after(b, p)
        if p > 0 then return 2 end
        return b == "char" and 1 or 2
    end
    local function pointee_size(ty)
        if ty.arr then return after(ty.b, ty.p) end
        return after(ty.b, ty.p - 1)
    end
    local function decay(ty)
        if ty.arr then return T(ty.b, ty.p + 1) end
        return ty
    end
    local function is_ptr(ty) return ty.p > 0 or ty.arr ~= nil end

    local function load_into(reg, areg, ty)
        if ty.p == 0 and ty.b == "char" then
            emit("ldb " .. reg .. ", [" .. areg .. "+0]")
        else
            emit("ld " .. reg .. ", [" .. areg .. "+0]")
        end
    end
    local function store_from(reg, areg, ty)
        if ty.p == 0 and ty.b == "char" then
            emit("stb [" .. areg .. "+0], " .. reg)
        else
            emit("st [" .. areg .. "+0], " .. reg)
        end
    end

    -- превращает lvalue (адрес в r0) в значение в r0
    local function rv(ty, lv)
        if ty.arr then return decay(ty) end
        if lv then
            if ty.b == "void" and ty.p == 0 then cerr("void не значение") end
            load_into("r0", "r0", ty)
        end
        return ty
    end

    local function find_local(name)
        for i = #scopes, 1, -1 do
            local v = scopes[i][name]
            if v then return v end
        end
    end
    local function alloc_local(name, ty)
        local size = 2
        if ty.arr then
            size = ty.arr * after(ty.b, ty.p)
            if size % 2 == 1 then size = size + 1 end
        end
        fsize = fsize + size
        local v = { off = -fsize, ty = ty }
        scopes[#scopes][name] = v
        return v
    end

    local function str_label(bytes)
        local key = table.concat(bytes, ",")
        if strings[key] then return strings[key] end
        local l = "S" .. (#data + 1)
        local parts = {}
        for _, c in ipairs(bytes) do
            if c == 10 then parts[#parts + 1] = "\\n"
            elseif c == 9 then parts[#parts + 1] = "\\t"
            elseif c == 13 then parts[#parts + 1] = "\\r"
            elseif c == 92 then parts[#parts + 1] = "\\\\"
            elseif c == 34 then parts[#parts + 1] = '\\"'
            elseif c >= 32 and c ~= 127 then parts[#parts + 1] = string.char(c)
            else error("в строке нельзя символ с кодом " .. c, 0) end
        end
        data[#data + 1] = l .. ": .str \"" .. table.concat(parts) .. "\""
        strings[key] = l
        return l
    end

    local parse_expr, parse_unary, parse_binary

    local function emit_cmp(op)
        local lt, le = newlabel(), newlabel()
        if op == "==" or op == "!=" then
            emit("cmp r0, r1")
            emit((op == "==" and "je " or "jne ") .. lt)
        else
            emit("ldi r2, 0x8000")
            emit("xor r0, r0, r2")
            emit("xor r1, r1, r2")
            if op == "<" then emit("cmp r0, r1"); emit("jlt " .. lt)
            elseif op == ">=" then emit("cmp r0, r1"); emit("jge " .. lt)
            elseif op == ">" then emit("cmp r1, r0"); emit("jlt " .. lt)
            else emit("cmp r1, r0"); emit("jge " .. lt) end
        end
        emit("ldi r0, 0")
        emit("jmp " .. le)
        label(lt)
        emit("ldi r0, 1")
        label(le)
    end

    local ARITH = { ["+"] = "add", ["-"] = "sub", ["*"] = "mul", ["/"] = "div", ["%"] = "mod", ["&"] = "and", ["|"] = "or", ["^"] = "xor" }

    -- r0 = левое, r1 = правое
    local function emit_binop(op, lt, rt)
        if op == "<<" or op == ">>" then
            local lp, le = newlabel(), newlabel()
            label(lp)
            emit("jz r1, " .. le)
            emit((op == "<<" and "shl" or "shr") .. " r0, r0, 1")
            emit("addi r1, r1, -1")
            emit("jmp " .. lp)
            label(le)
            return lt
        end
        if op == "==" or op == "!=" or op == "<" or op == ">" or op == "<=" or op == ">=" then
            emit_cmp(op)
            return T("int")
        end
        if (op == "+" or op == "-") and is_ptr(lt) and not is_ptr(rt) then
            if pointee_size(lt) == 2 then emit("shl r1, r1, 1") end
            emit(ARITH[op] .. " r0, r0, r1")
            return decay(lt)
        end
        if op == "+" and is_ptr(rt) and not is_ptr(lt) then
            if pointee_size(rt) == 2 then emit("shl r0, r0, 1") end
            emit("add r0, r0, r1")
            return decay(rt)
        end
        if op == "-" and is_ptr(lt) and is_ptr(rt) then
            emit("sub r0, r0, r1")
            if pointee_size(lt) == 2 then emit("shr r0, r0, 1") end
            return T("int")
        end
        emit(ARITH[op] .. " r0, r0, r1")
        return T("int")
    end

    local function parse_call(name)
        local n = 0
        if not isop(")") then
            repeat
                local t, l = parse_expr()
                rv(t, l)
                emit("push r0")
                n = n + 1
            until not accept(",")
        end
        expect(")")
        emit("call f_" .. name)
        if n > 0 then emit("addi sp, sp, " .. (2 * n)) end
        local f = funcs[name]
        return f and f.ret or T("int"), false
    end

    local function parse_primary()
        local t = toks[tp]
        if t.t == "num" then
            tp = tp + 1
            emit("ldi r0, " .. t.v)
            return T("int"), false
        elseif t.t == "str" then
            tp = tp + 1
            local bytes = t.v
            -- соседние строки склеиваются
            while toks[tp].t == "str" do
                for _, c in ipairs(toks[tp].v) do bytes[#bytes + 1] = c end
                tp = tp + 1
            end
            emit("ldi r0, " .. str_label(bytes))
            return T("char", 1), false
        elseif t.t == "id" then
            tp = tp + 1
            if accept("(") then return parse_call(t.v) end
            local v = find_local(t.v)
            if v then
                emit(("addi r0, r6, %d"):format(v.off))
                return v.ty, true
            end
            local g = globals[t.v]
            if g then
                emit("ldi r0, G_" .. t.v)
                return g.ty, true
            end
            tp = tp - 1
            cerr("неизвестное имя")
        elseif isop("(") then
            tp = tp + 1
            if is_type_start() then
                local ty = parse_type()
                expect(")")
                local t2, l2 = parse_unary()
                t2 = rv(t2, l2)
                if ty.p == 0 and ty.b == "char" then
                    emit("ldi r1, 255")
                    emit("and r0, r0, r1")
                end
                return ty, false
            end
            local ty, lv = parse_expr()
            expect(")")
            return ty, lv
        end
        cerr("ожидалось выражение")
    end

    local function parse_postfix()
        local ty, lv = parse_primary()
        while true do
            if isop("[") then
                tp = tp + 1
                ty = rv(ty, lv)
                if not is_ptr(ty) then cerr("индексировать можно только указатель") end
                emit("push r0")
                local it, il = parse_expr()
                rv(it, il)
                expect("]")
                local sz = pointee_size(ty)
                if sz == 2 then emit("shl r0, r0, 1") end
                emit("pop r1")
                emit("add r0, r1, r0")
                ty = T(ty.b, ty.p - 1)
                lv = true
            elseif isop("++") or isop("--") then
                if not lv then cerr("нужен lvalue") end
                local op = toks[tp].v
                tp = tp + 1
                local step = is_ptr(ty) and pointee_size(ty) or 1
                emit("mov r1, r0")
                load_into("r0", "r1", ty)
                emit(("addi r2, r0, %d"):format(op == "++" and step or -step))
                store_from("r2", "r1", ty)
                lv = false
            else
                break
            end
        end
        return ty, lv
    end

    function parse_unary()
        if isop("-") then
            tp = tp + 1
            local t, l = parse_unary()
            rv(t, l)
            emit("ldi r1, 0")
            emit("sub r0, r1, r0")
            return T("int"), false
        elseif isop("!") then
            tp = tp + 1
            local t, l = parse_unary()
            rv(t, l)
            local lt, le = newlabel(), newlabel()
            emit("jz r0, " .. lt)
            emit("ldi r0, 0")
            emit("jmp " .. le)
            label(lt)
            emit("ldi r0, 1")
            label(le)
            return T("int"), false
        elseif isop("~") then
            tp = tp + 1
            local t, l = parse_unary()
            rv(t, l)
            emit("ldi r1, 0xFFFF")
            emit("xor r0, r0, r1")
            return T("int"), false
        elseif isop("*") then
            tp = tp + 1
            local t, l = parse_unary()
            t = rv(t, l)
            if t.p == 0 then cerr("разыменовать можно только указатель") end
            return T(t.b, t.p - 1), true
        elseif isop("&") then
            tp = tp + 1
            local t, l = parse_unary()
            if not l then cerr("взять адрес можно только у lvalue") end
            if t.arr then return decay(t), false end
            return T(t.b, t.p + 1), false
        elseif isop("++") or isop("--") then
            local op = toks[tp].v
            tp = tp + 1
            local t, l = parse_unary()
            if not l then cerr("нужен lvalue") end
            local step = is_ptr(t) and pointee_size(t) or 1
            emit("mov r1, r0")
            load_into("r0", "r1", t)
            emit(("addi r0, r0, %d"):format(op == "++" and step or -step))
            store_from("r0", "r1", t)
            return t, false
        elseif isop("sizeof") then
            tp = tp + 1
            expect("(")
            local ty = parse_type()
            expect(")")
            emit("ldi r0, " .. ((ty.p == 0 and ty.b == "char") and 1 or 2))
            return T("int"), false
        end
        return parse_postfix()
    end

    local PREC = { ["||"] = 1, ["&&"] = 2, ["|"] = 3, ["^"] = 4, ["&"] = 5, ["=="] = 6, ["!="] = 6, ["<"] = 7, [">"] = 7, ["<="] = 7, [">="] = 7, ["<<"] = 8, [">>"] = 8, ["+"] = 9, ["-"] = 9, ["*"] = 10, ["/"] = 10, ["%"] = 10 }

    function parse_binary(minp)
        local ty, lv = parse_unary()
        while true do
            local t = toks[tp]
            local p = t.t == "op" and PREC[t.v]
            if not p or p < minp then break end
            local op = t.v
            tp = tp + 1
            ty = rv(ty, lv)
            lv = false
            if op == "&&" or op == "||" then
                local lf, le = newlabel(), newlabel()
                emit((op == "&&" and "jz r0, " or "jnz r0, ") .. lf)
                local rt, rl = parse_binary(p + 1)
                rv(rt, rl)
                emit((op == "&&" and "jz r0, " or "jnz r0, ") .. lf)
                emit("ldi r0, " .. (op == "&&" and 1 or 0))
                emit("jmp " .. le)
                label(lf)
                emit("ldi r0, " .. (op == "&&" and 0 or 1))
                label(le)
                ty = T("int")
            else
                emit("push r0")
                local rt, rl = parse_binary(p + 1)
                rt = rv(rt, rl)
                emit("mov r1, r0")
                emit("pop r0")
                ty = emit_binop(op, ty, rt)
            end
        end
        return ty, lv
    end

    local ASSIGN = { ["="] = true, ["+="] = "+", ["-="] = "-", ["*="] = "*", ["/="] = "/", ["%="] = "%", ["&="] = "&", ["|="] = "|", ["^="] = "^", ["<<="] = "<<", [">>="] = ">>" }

    function parse_expr()
        local ty, lv = parse_binary(1)
        local t = toks[tp]
        if t.t == "op" and ASSIGN[t.v] then
            if not lv or ty.arr then cerr("слева от присваивания нужен lvalue") end
            tp = tp + 1
            local compound = ASSIGN[t.v]
            emit("push r0")
            local rt, rl = parse_expr()
            rt = rv(rt, rl)
            if compound ~= true then
                emit("mov r2, r0")
                emit("pop r1")
                emit("push r1")
                load_into("r0", "r1", ty)
                emit("mov r1, r2")
                emit_binop(compound, rv(ty, false), rt)
            end
            emit("pop r1")
            store_from("r0", "r1", ty)
            return ty, false
        end
        return ty, lv
    end

    local function expr_stmt()
        local t, l = parse_expr()
        rv(t, l)
    end

    local parse_stmt

    local function parse_block()
        expect("{")
        scopes[#scopes + 1] = {}
        while not isop("}") do
            if toks[tp].t == "eof" then cerr("не закрыта скобка") end
            parse_stmt()
        end
        tp = tp + 1
        scopes[#scopes] = nil
    end

    local function parse_local_decl()
        local basety = parse_type()
        repeat
            local stars = 0
            while accept("*") do stars = stars + 1 end
            local ty = T(basety.b, basety.p + stars)
            local name = expect_id()
            if accept("[") then
                local n = toks[tp]
                if n.t ~= "num" then cerr("размер массива должен быть числом") end
                tp = tp + 1
                expect("]")
                ty = T(ty.b, ty.p, n.v)
            end
            local v = alloc_local(name, ty)
            if accept("=") then
                local t, l = parse_expr()
                rv(t, l)
                if ty.p == 0 and ty.b == "char" then
                    emit(("stb [r6+%d], r0"):format(v.off))
                else
                    emit(("st [r6+%d], r0"):format(v.off))
                end
            end
        until not accept(",")
        expect(";")
    end

    function parse_stmt()
        if isop("{") then
            parse_block()
        elseif is_type_start() then
            parse_local_decl()
        elseif accept("if") then
            expect("(")
            local t, l = parse_expr()
            rv(t, l)
            expect(")")
            local lelse, lend = newlabel(), newlabel()
            emit("jz r0, " .. lelse)
            parse_stmt()
            if accept("else") then
                emit("jmp " .. lend)
                label(lelse)
                parse_stmt()
                label(lend)
            else
                label(lelse)
            end
        elseif accept("while") then
            expect("(")
            local lc, le = newlabel(), newlabel()
            label(lc)
            local t, l = parse_expr()
            rv(t, l)
            expect(")")
            emit("jz r0, " .. le)
            loops[#loops + 1] = { brk = le, cont = lc }
            parse_stmt()
            loops[#loops] = nil
            emit("jmp " .. lc)
            label(le)
        elseif accept("for") then
            expect("(")
            scopes[#scopes + 1] = {}
            if not accept(";") then
                if is_type_start() then parse_local_decl() else expr_stmt(); expect(";") end
            end
            local lc, ls, le, lb = newlabel(), newlabel(), newlabel(), newlabel()
            label(lc)
            if not isop(";") then
                local t, l = parse_expr()
                rv(t, l)
                emit("jz r0, " .. le)
            end
            expect(";")
            emit("jmp " .. lb)
            label(ls)
            if not isop(")") then expr_stmt() end
            expect(")")
            emit("jmp " .. lc)
            label(lb)
            loops[#loops + 1] = { brk = le, cont = ls }
            parse_stmt()
            loops[#loops] = nil
            emit("jmp " .. ls)
            label(le)
            scopes[#scopes] = nil
        elseif accept("return") then
            if not isop(";") then
                local t, l = parse_expr()
                rv(t, l)
            end
            expect(";")
            emit("jmp " .. retlabel)
        elseif accept("break") then
            expect(";")
            if #loops == 0 then cerr("break вне цикла") end
            emit("jmp " .. loops[#loops].brk)
        elseif accept("continue") then
            expect(";")
            if #loops == 0 then cerr("continue вне цикла") end
            emit("jmp " .. loops[#loops].cont)
        elseif accept(";") then
            -- пустой оператор, ничего не делаем и гордимся
        else
            expr_stmt()
            expect(";")
        end
    end

    local function parse_function(rty, name)
        expect("(")
        local params = {}
        if not isop(")") then
            if isop("void") and toks[tp + 1].v == ")" then
                tp = tp + 1
            else
                repeat
                    local pt = parse_type()
                    local pn = expect_id()
                    if accept("[") then
                        accept("]")
                        pt = T(pt.b, pt.p + 1)
                    end
                    params[#params + 1] = { name = pn, ty = pt }
                until not accept(",")
            end
        end
        expect(")")
        funcs[name] = { ret = rty }
        if accept(";") then return end
        local saved = cur
        cur = {}
        scopes = { {} }
        fsize = 0
        retlabel = "f_" .. name .. "_ret"
        for i, p in ipairs(params) do
            scopes[1][p.name] = { off = 4 + 2 * (#params - i), ty = p.ty }
        end
        parse_block()
        local body = cur
        cur = {}
        label("f_" .. name)
        emit("push r6")
        emit("mov r6, sp")
        if fsize > 0 then emit("addi sp, sp, " .. (-fsize)) end
        for _, l in ipairs(body) do cur[#cur + 1] = l end
        label(retlabel)
        emit("mov sp, r6")
        emit("pop r6")
        emit("ret")
        local fl = optimize(cur)
        cur = saved
        for _, l in ipairs(fl) do cur[#cur + 1] = l end
    end

    local gdata = {}
    local function parse_global(basety, first_name)
        local name = first_name
        while true do
            local ty = basety
            if accept("[") then
                local n = toks[tp]
                if n.t ~= "num" then cerr("размер массива должен быть числом") end
                tp = tp + 1
                expect("]")
                ty = T(ty.b, ty.p, n.v)
            end
            local init = "0"
            if accept("=") then
                local t = toks[tp]
                if t.t == "num" then
                    init = tostring(t.v); tp = tp + 1
                elseif t.t == "str" then
                    init = str_label(t.v); tp = tp + 1
                elseif t.t == "op" and t.v == "-" and toks[tp + 1].t == "num" then
                    init = tostring(65536 - toks[tp + 1].v); tp = tp + 2
                else
                    cerr("начальное значение глобальной переменной должно быть константой")
                end
            end
            globals[name] = { ty = ty }
            gdata[#gdata + 1] = { name = name, ty = ty, init = init }
            if not accept(",") then break end
            local stars = 0
            while accept("*") do stars = stars + 1 end
            basety = T(basety.b, basety.p)
            name = expect_id()
        end
        expect(";")
    end

    -- ---- верхний уровень ----
    while toks[tp].t ~= "eof" do
        local ty = parse_type()
        local name = expect_id()
        if isop("(") then
            parse_function(ty, name)
        else
            parse_global(ty, name)
        end
    end
    if not funcs.main or not funcs.main.ret then error("нет функции main", 0) end

    local asm = {
        "    ldi sp, 0xE000",
        "    ldi r5, 0",
        "    call f_main",
        "    hlt",
        "f_putchar:",
        "    push r6",
        "    mov r6, sp",
        "pc_wait:",
        "    ldi r2, 0xFF01",
        "    ldb r3, [r2+0]",
        "    jnz r3, pc_wait",
        "    ld r1, [r6+4]",
        "    ldi r2, 0xFF00",
        "    stb [r2+0], r1",
        "    pop r6",
        "    ret",
        "f_getchar:",
        "    ldi r2, 0xFF03",
        "    ldb r0, [r2+0]",
        "    ret",
    }
    for _, l in ipairs(cur) do asm[#asm + 1] = l end
    for _, l in ipairs(data) do asm[#asm + 1] = l end
    for _, g in ipairs(gdata) do
        if g.ty.arr then
            local bytes = g.ty.arr * after(g.ty.b, g.ty.p)
            asm[#asm + 1] = "G_" .. g.name .. ":"
            local k = 0
            while k < bytes do
                local n = math.min(16, bytes - k)
                asm[#asm + 1] = "    .byte " .. string.rep("0", n, ",")
                k = k + n
            end
        else
            asm[#asm + 1] = "G_" .. g.name .. ": .word " .. g.init
        end
    end
    return table.concat(asm, "\n") .. "\n"
end

return compile(C_SOURCE)
