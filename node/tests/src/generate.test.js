const dsl = require('balena-configuration-dsl');

test('compilation of empty schema', () => {
    expect(
        dsl.generate_ui(`
            version: 1
            title: Empty schema
        ` )
    ).toEqual(
        {
            "json_schema": {
                "$$version": 1,
                "$schema": "http://json-schema.org/draft-04/schema#",
                "title": "Empty schema",
                "type": "object"
            },
            "ui_object": {}
        }
    );
});