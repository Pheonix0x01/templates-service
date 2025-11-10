CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE templates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    template_code TEXT NOT NULL,
    version INTEGER NOT NULL,
    type TEXT NOT NULL CHECK(type IN ('email_html', 'push_json')),
    language TEXT NOT NULL,
    content TEXT NOT NULL,
    created_by UUID NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN DEFAULT true NOT NULL,
    meta JSONB NULL,
    CONSTRAINT unique_template_version_language UNIQUE (template_code, version, language)
);

CREATE INDEX idx_templates_template_code ON templates(template_code);
CREATE INDEX idx_templates_language ON templates(language);
CREATE INDEX idx_templates_is_active ON templates(is_active);
CREATE INDEX idx_templates_created_at ON templates(created_at DESC);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_templates_updated_at
    BEFORE UPDATE ON templates
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();