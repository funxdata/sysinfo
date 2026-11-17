// deno-lint-ignore-file no-explicit-any
/**
 * sys_info FFI 测试（Deno）
 *
 * 通过 Deno.dlopen 加载动态库并调用导出的 post_* 函数。
 * 动态库优先取 dytes test 演练产物（.dytes/run-<pid>/dytes-test-<ts>/libary/），
 * 其次取环境变量 SYSINFO_LIB，最后回退 target/release 本地构建产物。
 *
 * 运行方式（在 ts/ 目录下）：
 *   deno task check          # 构建 + dytes test --keep + 本测试
 *   deno task test           # 仅运行本测试（要求动态库已存在）
 */

const MAX_OUTPUT_LEN = 65536;

const LIB_NAMES = Deno.build.os === "windows"
  ? { dytes: "sysinfo.dll", local: "sys_info.dll" }
  : Deno.build.os === "darwin"
  ? { dytes: "sysinfo.dylib", local: "libsys_info.dylib" }
  : { dytes: "sysinfo.so", local: "libsys_info.so" };

/** 项目根目录（ts/ 的上一级） */
const ROOT = (() => {
  let path = decodeURIComponent(new URL("..", import.meta.url).pathname);
  if (Deno.build.os === "windows") path = path.replace(/^\//, "");
  return path.replace(/[\\/]$/, "");
})();

function joinPath(...parts: string[]): string {
  return parts.join("/").replace(/\/+/g, "/");
}

function statSafe(path: string): Deno.FileInfo | null {
  try {
    return Deno.statSync(path);
  } catch {
    return null;
  }
}

function isFile(path: string): boolean {
  return statSafe(path)?.isFile ?? false;
}

/** 在 .dytes 的演练目录 / env 中查找 dytes 映射后的动态库 */
function findDytesLib(): string | null {
  const dytesDir = joinPath(ROOT, ".dytes");
  if (!statSafe(dytesDir)?.isDirectory) return null;

  const candidates: string[] = [];

  for (const run of Deno.readDirSync(dytesDir)) {
    if (!run.isDirectory || !run.name.startsWith("run-")) continue;
    const runDir = joinPath(dytesDir, run.name);
    for (const test of Deno.readDirSync(runDir)) {
      if (!test.isDirectory) continue;
      candidates.push(joinPath(runDir, test.name, "libary", LIB_NAMES.dytes));
    }
  }

  candidates.push(joinPath(dytesDir, "env", "libary", LIB_NAMES.dytes));

  let best: { path: string; mtime: number } | null = null;
  for (const path of candidates) {
    if (!isFile(path)) continue;
    const mtime = statSafe(path)?.mtime?.getTime() ?? 0;
    if (!best || mtime > best.mtime) best = { path, mtime };
  }
  return best?.path ?? null;
}

function findLib(): string {
  let fromEnv: string | undefined;
  try {
    fromEnv = Deno.env.get("SYSINFO_LIB");
  } catch {
    // 未授予 --allow-env 时忽略
  }
  if (fromEnv) {
    if (!isFile(fromEnv)) {
      throw new Error(`SYSINFO_LIB 指向的文件不存在: ${fromEnv}`);
    }
    return fromEnv;
  }

  const dytesLib = findDytesLib();
  if (dytesLib) return dytesLib;

  const localLib = joinPath(ROOT, "target", "release", LIB_NAMES.local);
  if (isFile(localLib)) return localLib;

  throw new Error(
    "找不到 sysinfo 动态库。请先运行 deno task check（构建并执行 dytes test --keep），" +
      `或设置 SYSINFO_LIB 指向动态库。已查找: ${localLib}`,
  );
}

const FN_NAMES = [
  "post_create_dir",
  "post_cur_dir",
  "post_file_exists",
  "post_file_info",
  "post_component_info",
  "post_cpu_info",
  "post_disk_info",
  "post_memory_info",
  "post_network_info",
  "post_process_list",
  "post_system_basic_info",
] as const;

type FnName = typeof FN_NAMES[number];

const symbols = Object.fromEntries(
  FN_NAMES.map((name) => [
    name,
    { parameters: ["pointer", "pointer"], result: "i32" },
  ]),
) as unknown as Deno.ForeignLibraryInterface;

const libPath = findLib();
const lib = Deno.dlopen(libPath, symbols);
globalThis.addEventListener("unload", () => lib.close());

const encoder = new TextEncoder();
const decoder = new TextDecoder();

type FfiFn = (
  input: Deno.PointerValue,
  out: Deno.PointerValue,
) => number;

function assert(condition: unknown, message = "断言失败"): asserts condition {
  if (!condition) throw new Error(message);
}

function assertEquals(actual: unknown, expected: unknown, message?: string) {
  if (actual !== expected) {
    throw new Error(message ?? `期望 ${expected}，实际 ${actual}`);
  }
}

/** 调用 FFI，输入为已编码的 JSON 字节；null 表示空指针 */
function invoke(name: FnName, input: Uint8Array | null): any {
  const out = new Uint8Array(MAX_OUTPUT_LEN);
  const outPtr = Deno.UnsafePointer.of(out);
  const inPtr = input === null ? null : Deno.UnsafePointer.of(input);
  const ret = (lib.symbols[name] as unknown as FfiFn)(inPtr, outPtr);
  assertEquals(ret, 0, `${name} 返回码应为 0，实际 ${ret}`);

  const end = out.indexOf(0);
  const text = decoder.decode(out.subarray(0, end < 0 ? out.length : end));
  try {
    return JSON.parse(text);
  } catch {
    throw new Error(`${name} 输出不是合法 JSON: ${text}`);
  }
}

function call(name: FnName, input?: unknown): any {
  const bytes = input === undefined
    ? null
    : encoder.encode(`${JSON.stringify(input)}\0`);
  return invoke(name, bytes);
}

function callRaw(name: FnName, raw: string): any {
  return invoke(name, encoder.encode(`${raw}\0`));
}

Deno.test("系统基础信息", () => {
  const v = call("post_system_basic_info");
  assertEquals(v.code, 200);
  assertEquals(typeof v.data.uptime, "number");
  assertEquals(typeof v.data.boot_time, "number");
  assertEquals(typeof v.data.load_average, "object");
});

Deno.test("内存信息", () => {
  const v = call("post_memory_info");
  assertEquals(v.code, 200);

  assert(v.data.total_bytes > 0, "总内存应大于 0");
  assert(v.data.used_bytes <= v.data.total_bytes, "已用内存不应超过总量");
  assert(v.data.free_bytes <= v.data.total_bytes, "空闲内存不应超过总量");
  assert(
    v.data.usage_percent >= 0 && v.data.usage_percent <= 100,
    "usage_percent 应在 0..=100",
  );
  assertEquals(typeof v.data.total_swap_bytes, "number");
  assertEquals(typeof v.data.used_swap_bytes, "number");
});

Deno.test("CPU 信息", () => {
  const v = call("post_cpu_info");
  assertEquals(v.code, 200);

  assert(v.data.logical_core_count > 0, "逻辑核心数应大于 0");
  assertEquals(v.data.cpus.length, v.data.logical_core_count);
  for (const cpu of v.data.cpus) {
    assertEquals(typeof cpu.name, "string");
    assertEquals(typeof cpu.brand, "string");
    assertEquals(typeof cpu.frequency, "number");
    assertEquals(typeof cpu.usage, "number");
  }
});

Deno.test("磁盘信息", () => {
  const v = call("post_disk_info");
  assertEquals(v.code, 200);

  assert(v.data.count > 0, "至少应检测到一个磁盘");
  assertEquals(v.data.disks.length, v.data.count);
  for (const disk of v.data.disks) {
    assertEquals(typeof disk.name, "string");
    assertEquals(typeof disk.mount_point, "string");
    assertEquals(typeof disk.file_system, "string");
    assertEquals(typeof disk.kind, "string");

    const total = disk.total_bytes as number;
    const used = disk.used_bytes as number;
    const available = disk.available_bytes as number;
    assert(used <= total, "已用空间不应超过总量");
    assert(available <= total, "可用空间不应超过总量");
    assert(
      disk.usage_percent >= 0 && disk.usage_percent <= 100,
      "usage_percent 应在 0..=100",
    );
  }
});

Deno.test("网络接口信息", () => {
  const v = call("post_network_info");
  assertEquals(v.code, 200);
  assert(Array.isArray(v.data.interfaces), "interfaces 应为数组");
  assertEquals(v.data.interfaces.length, v.data.count);
  for (const iface of v.data.interfaces) {
    assertEquals(typeof iface.interface, "string");
    assertEquals(typeof iface.mac_address, "string");
    assertEquals(typeof iface.total_received, "number");
    assertEquals(typeof iface.total_transmitted, "number");
  }
});

Deno.test("进程列表与分页", () => {
  const all = call("post_process_list");
  assertEquals(all.code, 200);
  assert(all.data.count > 0, "进程数量应大于 0");
  assertEquals(all.data.offset, 0);
  assertEquals(all.data.limit, 100);
  assert(Array.isArray(all.data.processes), "processes 应为数组");

  const page = call("post_process_list", { limit: 5, offset: 0 });
  assertEquals(page.code, 200);
  assertEquals(page.data.limit, 5);
  assertEquals(page.data.offset, 0);
  assert(page.data.count <= 5, "分页 count 不应超过 limit");
  assertEquals(page.data.processes.length, page.data.count);
  assertEquals(typeof page.data.has_more, "boolean");
});

Deno.test("温度组件信息", () => {
  const v = call("post_component_info");
  assertEquals(v.code, 200);
  assert(Array.isArray(v.data.components), "components 应为数组");
  assertEquals(v.data.components.length, v.data.count);
});

Deno.test("文件：当前目录与目录创建", () => {
  const cur = call("post_cur_dir");
  assertEquals(cur.code, 200);
  assert(
    typeof cur.data.path === "string" &&
      (cur.data.path.startsWith("/") || /^[A-Za-z]:[\\/]/.test(cur.data.path)),
    "当前目录应为绝对路径",
  );

  const name = `ffi_ts_${Date.now()}_${
    Math.random().toString(16).slice(2)
  }`;
  const created = call("post_create_dir", { dir_name: name });
  assertEquals(created.code, 200);

  const path = created.data.path as string;
  try {
    assert(statSafe(path)?.isDirectory, "目录应存在");
    assertEquals(call("post_file_exists", { path }).data.exists, true);

    const info = call("post_file_info", { path });
    assertEquals(info.code, 200);
    assertEquals(info.data.exists, true);
    assertEquals(info.data.is_dir, true);
    assertEquals(info.data.is_file, false);
  } finally {
    Deno.removeSync(path);
  }

  assertEquals(call("post_file_exists", { path }).data.exists, false);
});

Deno.test("文件：临时目录元数据", () => {
  const dir = Deno.makeTempDirSync({ prefix: "sysinfo-ffi-" });
  try {
    const info = call("post_file_info", { path: dir });
    assertEquals(info.code, 200);
    assertEquals(info.data.exists, true);
    assertEquals(info.data.is_dir, true);
    assertEquals(info.data.is_file, false);
    assertEquals(info.data.is_symlink, false);
  } finally {
    Deno.removeSync(dir);
  }

  const missing = call("post_file_info", {
    path: joinPath(Deno.cwd(), "no-such-path-xyz-123"),
  });
  assertEquals(missing.code, 200);
  assertEquals(missing.data.exists, false);
});

Deno.test("错误码：空指针、非法 JSON、非法路径", () => {
  // 空指针
  assertEquals(invoke("post_create_dir", null).code, 400100);
  // 非法 JSON
  assertEquals(callRaw("post_create_dir", "{bad}").code, 400102);
  assertEquals(callRaw("post_cur_dir", "{bad}").code, 400102);
  // 缺少字段
  assertEquals(call("post_create_dir", { other: "x" }).code, 400102);
  // 空目录名
  assertEquals(call("post_create_dir", { dir_name: "   " }).code, 400103);
  // 路径穿越
  assertEquals(call("post_create_dir", { dir_name: "../evil" }).code, 400104);
  // 绝对路径
  const absolute = Deno.build.os === "windows" ? "C:\\evil" : "/tmp/evil";
  assertEquals(
    call("post_create_dir", { dir_name: absolute }).code,
    400104,
  );
});
