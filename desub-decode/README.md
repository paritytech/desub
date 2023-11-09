# Desub-decoder

A library for decoding bytes given type information.

In modern times (i.e. runtimes with V14 metadata or beyond), the type information is provided by a `scale_info::PortableRegistry` and we can lean on [existing decoding infrastructure](https://github.com/paritytech/scale-decode) for that.

In the olden days, the metadata (V13 and below) would mention the _names_ of types, but give no specific information on their structure. Thus, we have to manually provide mappings from name to type description in order to know how to decode things.

This library provides a trait which abstracts over these differences, and implementations for old and new type information. It's essentially a more general version of [`scale-decode`](https://github.com/paritytech/scale-decode), capable of working with historic and new type information.