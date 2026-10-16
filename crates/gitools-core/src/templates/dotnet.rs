use super::common::{ci_dotnet, file, gitignore, opt_ci, opt_editorconfig, opt_gitignore, opt_license, opt_readme};
use crate::template::{Template, TemplateOption};

pub fn template() -> Template {
    Template {
        id: "dotnet".into(),
        name: ".NET".into(),
        tagline: "C# — konsola albo minimalne API".into(),
        description: "Rozwiązanie .NET 9 z podziałem na projekty, xUnit, formatowaniem \
                      i potokiem GitHub Actions."
            .into(),
        icon: "🟣".into(),
        accent: "#512bd4".into(),
        tags: vec!["csharp".into(), "dotnet".into(), "backend".into()],
        options: vec![
            TemplateOption::choice(
                "kind",
                "Typ projektu",
                "Aplikacja konsolowa albo minimalne API",
                "Projekt",
                &[("console", "Konsola"), ("web", "Minimalne API")],
                "console",
            ),
            TemplateOption::toggle(
                "tests",
                "Testy xUnit",
                "Projekt testowy z xUnit i FluentAssertions",
                "Jakość",
                true,
            ),
            TemplateOption::toggle(
                "dockerfile",
                "Dockerfile",
                "Kontener z obrazem .NET SDK",
                "Dystrybucja",
                false,
            ),
            opt_ci("dotnet test"),
            opt_readme(),
            opt_gitignore(),
            opt_editorconfig(),
            opt_license(),
        ],
        hints: vec!["dotnet restore".into(), "dotnet run --project src/{{project_pascal}}".into()],
        files: vec![
            file(
                "{{project_slug}}.slnx",
                r#"<Solution>
  <Folder Name="/src/">
    <Project Path="src/{{project_pascal}}/{{project_pascal}}.csproj" />
  </Folder>
{{#if tests}}
  <Folder Name="/tests/">
    <Project Path="tests/{{project_pascal}}.Tests/{{project_pascal}}.Tests.csproj" />
  </Folder>
{{/if}}
</Solution>
"#,
            ),
            file(
                "Directory.Build.props",
                r#"<Project>
  <PropertyGroup>
    <TargetFramework>net9.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
    <TreatWarningsAsErrors>true</TreatWarningsAsErrors>
    <InvariantGlobalization>true</InvariantGlobalization>
  </PropertyGroup>
</Project>
"#,
            ),
            file(
                "src/{{project_pascal}}/{{project_pascal}}.csproj",
                r#"<Project Sdk="Microsoft.NET.Sdk">
{{#if kind=web}}
  <PropertyGroup>
    <Web>true</Web>
  </PropertyGroup>

  <ItemGroup>
    <PackageReference Include="Microsoft.AspNetCore.OpenApi" Version="9.0.0" />
    <PackageReference Include="Swashbuckle.AspNetCore" Version="7.2.0" />
  </ItemGroup>
{{/if}}
{{#unless license=none}}
  <ItemGroup>
    <PackageReference Include="System.CommandLine" Version="2.0.0-beta4.22272.1" Condition="'$(Configuration)' == 'Release'" />
  </ItemGroup>
{{/unless}}
</Project>
"#,
            ),
            file(
                "src/{{project_pascal}}/Program.cs",
                r#"// {{project_name}} - {{description}}
{{#if kind=console}}
var name = args.Length > 0 ? args[0] : "świecie";

Console.WriteLine($"Cześć, {name}!");
Console.WriteLine($"{{project_name}} v{typeof(Program).Assembly.GetName().Version}");
{{/if}}
{{#if kind=web}}
var builder = WebApplication.CreateBuilder(args);
var app = builder.Build();

app.MapGet("/health", () => Results.Ok(new
{
    status = "ok",
    service = "{{project_slug}}",
    version = typeof(Program).Assembly.GetName().Version?.ToString(),
}));

app.Run();
{{/if}}

/// <summary>Punkt wejścia aplikacji.</summary>
public partial class Program;
"#,
            ),
            file(
                "src/{{project_pascal}}/appsettings.json",
                r#"{
  "Logging": {
    "LogLevel": {
      "Default": "Information",
      "Microsoft.AspNetCore": "Warning"
    }
  },
  "AllowedHosts": "*"
}
"#
            )
            .when("kind=web"),
            file(
                "tests/{{project_pascal}}.Tests/{{project_pascal}}.Tests.csproj",
                r#"<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <IsPackable>false</IsPackable>
  </PropertyGroup>

  <ItemGroup>
    <PackageReference Include="Microsoft.NET.Test.Sdk" Version="17.12.0" />
    <PackageReference Include="xunit" Version="2.9.2" />
    <PackageReference Include="xunit.runner.visualstudio" Version="2.8.2" />
    <PackageReference Include="FluentAssertions" Version="6.12.2" />
  </ItemGroup>

  <ItemGroup>
    <ProjectReference Include="..\..\src\{{project_pascal}}\{{project_pascal}}.csproj" />
  </ItemGroup>

</Project>
"#,
            )
            .when("tests"),
            file(
                "tests/{{project_pascal}}.Tests/SmokeTests.cs",
                r#"using FluentAssertions;

namespace {{project_pascal}}.Tests;

public class SmokeTests
{
    [Fact]
    public void NazwaProjektu_JestNiepusta()
    {
        // arrange
        var assembly = typeof(global::Program).Assembly;

        // act
        var name = assembly.GetName().Name;

        // assert
        name.Should().NotBeNullOrWhiteSpace();
    }
}
"#,
            )
            .when("tests"),
            file(
                "Dockerfile",
                r#"FROM mcr.microsoft.com/dotnet/sdk:9.0 AS build
WORKDIR /src

COPY . .
RUN dotnet restore {{project_slug}}.slnx
RUN dotnet publish "src/{{project_pascal}}/{{project_pascal}}.csproj" -c Release -o /app --no-restore

FROM mcr.microsoft.com/dotnet/aspnet:9.0 AS runtime
{{#if kind=console}}
WORKDIR /app
COPY --from=build /app ./
ENTRYPOINT ["dotnet", "{{project_pascal}}.dll"]
{{/if}}
{{#if kind=web}}
EXPOSE 8080
ENV ASPNETCORE_URLS=http://+:8080
WORKDIR /app
COPY --from=build /app ./
ENTRYPOINT ["dotnet", "{{project_pascal}}.dll"]
{{/if}}
"#,
            )
            .when("dockerfile"),
            file(
                ".editorconfig",
                r#"root = true

[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true

[*.cs]
indent_style = space
indent_size = 4
csharp_new_line_before_open_brace = all
csharp_style_namespace_declarations = file_scoped:suggestion
dotnet_sort_system_directives_first = true

[*.{json,yml,yaml}]
indent_size = 2
"#
            )
            .when("editorconfig"),
            gitignore(
                &[
                    "bin/",
                    "obj/",
                    "*.user",
                    ".vs/",
                    "TestResults/",
                    "\n# edytory",
                    ".idea/",
                    ".vscode/*",
                    "!.vscode/extensions.json",
                ],
            )
            .when("gitignore"),
            ci_dotnet().when("ci"),
            file(
                "README.md",
                r#"# {{project_name}}

{{description}}

## Wymagania

- .NET SDK 9.0

## Start

```bash
dotnet restore
dotnet run --project src/{{project_pascal}}
```

## Testy

```bash
dotnet test
```

## Struktura

```
{{project_slug}}/
├── {{project_slug}}.slnx
├── src/{{project_pascal}}/
{{#if tests}}└── tests/{{project_pascal}}.Tests/
{{/if}}```

## Licencja

{{license_spdx}}
"#,
            )
            .when("readme"),
        ],
    }
}