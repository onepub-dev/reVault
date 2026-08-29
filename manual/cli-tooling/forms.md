---
description: Storing related values in a form
---

# Forms

Forms allow you to store a group of related values. You can perform many of the same operations that forms provide by using Environment Variables, but forms provide better ergonomics for managing related values.

At its heart a form contains a set of fields which are name/value pairs with a type. Instance of a form are stored as a form record. Like Environment Variables form records can be stored at path.

To create a form record you first have to define a Form:

```
lockbox form define 
```

