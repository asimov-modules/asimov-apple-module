{
  "@context": {
    "schema": "https://schema.org/",
    "xsd": "http://www.w3.org/2001/XMLSchema#",
    "name": {
      "@id": "schema:name",
      "@type": "xsd:string"
    },
    "givenName": {
      "@id": "schema:givenName",
      "@type": "xsd:string"
    },
    "familyName": {
      "@id": "schema:familyName",
      "@type": "xsd:string"
    },
    "telephone": {
      "@id": "schema:telephone",
      "@type": "schema:ContactPoint"
    },
    "source": {
      "@id": "schema:isBasedOn",
      "@type": "xsd:string"
    }
  },
  "@type": .["@type"],
  "@id": .["@id"],
  "name": .name,
  "source": .source,
  "givenName": .givenName,
  "familyName": .familyName,
  "telephone": .telephone
} | with_entries(select(.value != null))
