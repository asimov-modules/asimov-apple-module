{
  "@context": {
    "schema": "https://schema.org/",
    "xsd": "http://www.w3.org/2001/XMLSchema#",
    "name": {
      "@id": "schema:name",
      "@type": "xsd:string"
    },
    "startDate": {
      "@id": "schema:startDate",
      "@type": "xsd:string"
    },
    "endDate": {
      "@id": "schema:endDate",
      "@type": "xsd:string"
    },
    "location": {
      "@id": "schema:location",
      "@type": "xsd:string"
    },
    "description": {
      "@id": "schema:description",
      "@type": "xsd:string"
    },
    "isPartOf": {
      "@id": "schema:isPartOf",
      "@type": "xsd:string"
    },
    "source": {
      "@id": "schema:isBasedOn",
      "@type": "xsd:string"
    }
  },
  "@type": .["@type"],
  "@id": .["@id"],
  "name": .name,
  "startDate": .startDate,
  "endDate": .endDate,
  "location": .location,
  "description": .description,
  "isPartOf": .isPartOf,
  "source": .source
} | with_entries(select(.value != null))
