import { describe, it, expect } from "vitest";

describe("Test Float Bindings", () => {
  describe("CJS", async () => {
    await runTests("cjs");
  });

  describe("ESM", async () => {
    await runTests("esm");
  });

  async function runTests(mod: "cjs" | "esm") {
    const { Float } = await (mod === "cjs"
      ? import("../dist/cjs")
      : import("../dist/esm"));

    const ok = <T>(r: { value?: T; error?: unknown }): T => {
      expect(r.error).toBeUndefined();
      return r.value as T;
    };
    const p = (s: string) => ok<any>(Float.parse(s));
    const plain = (f: any) => ok<string>(f.formatWithScientific(false));
    const sci = (f: any) => ok<string>(f.formatWithScientific(true));
    const hex = (exponent: string, coefficient: string) =>
      `0x${exponent}${coefficient.padStart(56, coefficient.startsWith("f") ? "f" : "0")}`;

    it("parse packs the exponent and coefficient", () => {
      expect(p("3.14").asHex()).toBe(hex("fffffffe", "13a"));
      expect(p("-3.14").asHex()).toBe(hex("fffffffe", "fec6"));
      expect(p("1000").asHex()).toBe(hex("00000000", "3e8"));
    });

    it("parse reports each error selector", () => {
      const cases: [string, string][] = [
        ["abc", "There are no digits where the number should start."],
        ["1.", "The decimal point is not followed by any digits."],
        ["1e", "The exponent has no digits after the `e`."],
        [
          `1.${"1".repeat(80)}`,
          "The number has more precision than a Float can represent exactly.",
        ],
        ["1x", "There are unexpected characters after the number."],
      ];
      for (const [s, readable] of cases) {
        const r = Float.parse(s);
        expect(r.value).toBeUndefined();
        expect(r.error!.readableMsg).toBe(readable);
      }
    });

    it("fromHex and asHex round trip, rejecting bad hex", () => {
      const h = hex("fffffffe", "13a");
      expect(ok<any>(Float.fromHex(h)).asHex()).toBe(h);
      const bad = Float.fromHex("0x13a");
      expect(bad.value).toBeUndefined();
      expect(bad.error!.readableMsg).toBe(
        "The provided value is not a valid hex string: 0x13a",
      );
    });

    it("formatWithScientific formats both ways", () => {
      expect(plain(p("123.456"))).toBe("123.456");
      expect(sci(p("123.456"))).toBe("1.23456e2");
      expect(plain(p("0.00001"))).toBe("0.00001");
      expect(sci(p("0.00001"))).toBe("1e-5");
      expect(plain(p("-2500"))).toBe("-2500");
      expect(sci(p("-2500"))).toBe("-2.5e3");
      const r = p("1e1001").formatWithScientific(false);
      expect(r.value).toBeUndefined();
      expect(r.error!.readableMsg).toMatch(
        /^The Float operation failed with a decimal float error: UnformatableExponent/,
      );
    });

    it("fixed decimal conversions", () => {
      expect(plain(ok(Float.fromFixedDecimal(12345n, 2)))).toBe("123.45");
      expect(ok(p("123.45").toFixedDecimal(4))).toBe(1234500n);
      const lossy = Float.fromFixedDecimalLossy(1n + 10n ** 70n, 0);
      expect(lossy.lossless).toBe(false);
      expect(sci(lossy.float)).toBe("1e70");
      const exact = Float.fromFixedDecimalLossy(5n, 1);
      expect(exact.lossless).toBe(true);
      expect(plain(exact.float)).toBe("0.5");
      expect(ok(p("12.345").toFixedDecimalLossy(2))).toEqual({
        value: "1234",
        lossless: false,
      });
      expect(ok(p("12.34").toFixedDecimalLossy(2))).toEqual({
        value: "1234",
        lossless: true,
      });
      const negative = p("-1").toFixedDecimal(0);
      expect(negative.value).toBeUndefined();
      expect(negative.error!.readableMsg).toMatch(
        /decimal float error: NegativeFixedDecimalConversion/,
      );
      const lossyTo = p("0.5").toFixedDecimal(0);
      expect(lossyTo.error!.readableMsg).toMatch(/LossyConversionFromFloat/);
      const lossyFrom = Float.fromFixedDecimal(1n + 10n ** 70n, 0);
      expect(lossyFrom.error!.readableMsg).toMatch(/LossyConversionToFloat/);
    });

    it("bigint conversions are the raw 32 bytes", () => {
      expect(Float.fromBigint(18n).asHex()).toBe(hex("00000000", "12"));
      expect(ok<any>(Float.tryFromBigint(0x1234n)).asHex()).toBe(
        hex("00000000", "1234"),
      );
      const negOne = p("-1");
      expect(negOne.toBigint()).toBe(2n ** 224n - 1n);
      expect(ok(negOne.tryToBigint())).toBe(2n ** 224n - 1n);
      expect(p("3.14").toBigint()).toBe(BigInt(hex("fffffffe", "13a")));
    });

    it("constants are the documented extremes", () => {
      expect(ok<any>(Float.maxPositiveValue()).asHex()).toBe(
        `0x7fffffff7${"f".repeat(55)}`,
      );
      expect(ok<any>(Float.minPositiveValue()).asHex()).toBe(
        `0x80000000${"0".repeat(55)}1`,
      );
      expect(ok<any>(Float.maxNegativeValue()).asHex()).toBe(
        `0x80000000${"f".repeat(56)}`,
      );
      expect(ok<any>(Float.minNegativeValue()).asHex()).toBe(
        `0x7fffffff8${"0".repeat(55)}`,
      );
      expect(ok<any>(Float.zero()).asHex()).toBe(`0x${"0".repeat(64)}`);
    });

    it("comparisons", () => {
      const [one, two, ten, tenAgain] = [p("1"), p("2"), p("10"), p("1e1")];
      const all = (a: any, b: any) => [
        ok(a.lt(b)),
        ok(a.lte(b)),
        ok(a.eq(b)),
        ok(a.gte(b)),
        ok(a.gt(b)),
      ];
      expect(all(one, two)).toEqual([true, true, false, false, false]);
      expect(all(two, one)).toEqual([false, false, false, true, true]);
      expect(all(ten, tenAgain)).toEqual([false, true, true, true, false]);
      expect(ok<any>(one.min(two)).asHex()).toBe(one.asHex());
      expect(ok<any>(one.max(two)).asHex()).toBe(two.asHex());
      expect(ok(p("0").isZero())).toBe(true);
      expect(ok(p("-0.1").isZero())).toBe(false);
    });

    it("arithmetic", () => {
      const [a, b] = [p("3.14"), p("-1.04")];
      expect(plain(ok(a.add(b)))).toBe("2.1");
      expect(plain(ok(a.sub(b)))).toBe("4.18");
      expect(plain(ok(a.mul(b)))).toBe("-3.2656");
      expect(plain(ok(p("1").div(p("8"))))).toBe("0.125");
      expect(plain(ok(p("0.04").inv()))).toBe("25");
      expect(plain(ok(b.abs()))).toBe("1.04");
      expect(plain(ok(a.neg()))).toBe("-3.14");
      expect(plain(ok(p("-7.25").floor()))).toBe("-8");
      expect(plain(ok(p("-7.25").frac()))).toBe("-0.25");
      const byZero = p("1").div(p("0"));
      expect(byZero.value).toBeUndefined();
      expect(byZero.error!.readableMsg).toMatch(
        /^The Float operation failed with a decimal float error: DivisionByZero/,
      );
      const overflow = ok<any>(Float.maxPositiveValue()).mul(p("10"));
      expect(overflow.error!.readableMsg).toMatch(/ExponentOverflow/);
    });
  }
});
