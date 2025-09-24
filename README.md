# bunner_qs

A Rust port of the popular [qs](https://www.npmjs.com/package/qs) JavaScript library.

## Differences from original `qs`

This port intentionally omits several JavaScript-specific runtime value types that are either not representable or not commonly useful in typical Rust applications:

- No Function values: JavaScript functions (callables with dynamic properties) are not modeled. Any such values should be normalized before passing data into this library.
- No Symbol values: JavaScript Symbols are unique identity primitives; they are rarely (if ever) used in query strings and are excluded.
- No RegExp objects: Regular expression objects are not serialized or detected specially; treat them as plain strings prior to use.

Implications:
- Tests or behaviors in the original `qs` that exercised these types are removed or adjusted.
- Encoding logic only targets primitives (null, booleans, numbers, strings), arrays, and objects.
- Utility predicates like `isRegExp` / `isBuffer` only implement what is practical/relevant in Rust; `isRegExp` has been dropped alongside RegExp support.

If future requirements arise, these capabilities can be reintroduced behind optional feature flags without breaking the simplified core API.

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

This is a derivative work based on the [qs JavaScript library](https://github.com/ljharb/qs) (BSD-3-Clause License), with original license included in [LICENSE.md](LICENSE.md).
## References

- [qs on npm](https://www.npmjs.com/package/qs)
- [qs on GitHub](https://github.com/ljharb/qs)

## Acknowledgments

This library is inspired by and aims to be compatible with the JavaScript [qs](https://github.com/ljharb/qs) library.