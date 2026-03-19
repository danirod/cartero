<?xml version='1.0'?> <!--*- mode: nxml -*-->
<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
                version="1.0">

  <xsl:output method="html" encoding="ISO-8859-1" indent="yes" />
  <xsl:key name="module-id" match="moduleset/*" use="@id" />

  <xsl:template match="/">
    <html>
      <head>
        <title>Module Set</title>
        <style type="text/css">
          <xsl:text>
            dl {
              -moz-column-width: 40em;
              -moz-column-gap: 2em;
            }
            dt {
              font-weight: bold;
              background: #8f8;
              display: inline;
              padding: 0 1ex;
            }
            dt.metamodule {
              background: #F08080;
            }
            dt.tarball {
              background: #EEDD82;
            }
            dd {
              font-size: smaller;
            }
            dd {
              margin-bottom: 0.5em;
            }
            th {
              text-align: left;
              vertical-align: top;
            }
            ul.patches {
              list-style: none;
              padding: 0;
              margin: 0;
            }
          </xsl:text>
        </style>
      </head>
      <body>
        <xsl:apply-templates />
      </body>
    </html>
  </xsl:template>

  <xsl:template match="moduleset">
    <h1>JHBuild Module Set</h1>
    <dl>
    <xsl:apply-templates />
    </dl>
  </xsl:template>

  <xsl:template match="dependencies|suggests|after">
    <xsl:variable name="deps" select="dep/@package" />
    <xsl:for-each select="$deps">
      <a href="#{generate-id(key('module-id', .))}">
        <xsl:value-of select="." />
      </a>
      <xsl:if test="not($deps[last()] = .)">
        <xsl:text>, </xsl:text>
      </xsl:if>
    </xsl:for-each>
  </xsl:template>

  <xsl:template match="moduleset/*">
    <xsl:param name="reponame">
      <xsl:choose>
        <xsl:when test="branch/@repo"><xsl:value-of select="branch/@repo"/></xsl:when>
        <xsl:otherwise>
          <xsl:value-of select="//repository[@default = 'yes']/@name"/>
        </xsl:otherwise>
      </xsl:choose>
    </xsl:param>

    <dt id="{generate-id(.)}">
      <xsl:attribute name="class">module
        <xsl:if test="name(.) = 'tarball' or
                      //repository[@name = $reponame]/@type = 'tarball'">tarball</xsl:if>
      </xsl:attribute>
      <xsl:value-of select="@id" />
    </dt>

    <dd>
      <table>
        <tr>
          <th>Module:</th>
          <td>
            <xsl:choose>
              <xsl:when test="@module">
                <xsl:value-of select="@module" />
              </xsl:when>
              <xsl:otherwise>
                <xsl:value-of select="@id" />
              </xsl:otherwise>
            </xsl:choose>
            <xsl:if test="branch/@revision">
              <xsl:text> revision: </xsl:text>
              <xsl:value-of select="branch/@revision" />
            </xsl:if>
          </td>
        </tr>
        <xsl:if test="name(.) = 'tarball' or //repository[@name = $reponame]/@type = 'tarball'">
          <tr>
            <th>URL:</th>
            <td>
              <xsl:variable name="url">
                <xsl:choose>
                  <xsl:when test="name(.) = 'tarball'">
                    <xsl:value-of select="source/@href"/>
                  </xsl:when>
                  <xsl:otherwise>
                    <xsl:value-of select="//repository[@name=$reponame]/@href"
                     /><xsl:value-of select="branch/@module"/>
                  </xsl:otherwise>
                </xsl:choose>
              </xsl:variable>
              <a href="{$url}"><xsl:value-of select="$url"/></a>
              <xsl:if test="branch/@size"> (<xsl:value-of select="branch/@size"/> bytes)</xsl:if>
              <xsl:if test="source/@size"> (<xsl:value-of select="source/@size"/> bytes)</xsl:if>
            </td>
          </tr>
        </xsl:if>
        <xsl:if test="branch/patch">
          <tr>
            <th>Patches:</th>
            <td>
              <ul class="patches">
                <xsl:apply-templates select="branch/patch"/>
              </ul>
            </td>
          </tr>
        </xsl:if>
        <xsl:if test="@checkoutdir">
          <tr>
            <th>Checkout directory:</th>
            <td><xsl:value-of select="@checkoutdir" /></td>
          </tr>
        </xsl:if>
        <xsl:if test="@autogenargs">
          <tr>
            <th>Autogen args:</th>
            <td><xsl:value-of select="@autogenargs" /></td>
          </tr>
        </xsl:if>
        <xsl:if test="dependencies/dep">
          <tr>
            <th>Dependencies:</th>
            <td><xsl:apply-templates select="dependencies" /></td>
          </tr>
        </xsl:if>
        <xsl:if test="suggests">
          <tr>
            <th>Suggests:</th>
            <td><xsl:apply-templates select="suggests" /></td>
          </tr>
        </xsl:if>
        <xsl:if test="after">
          <tr>
            <th>After:</th>
            <td><xsl:apply-templates select="after" /></td>
          </tr>
        </xsl:if>
      </table>
    </dd>
  </xsl:template>

  <xsl:template match="moduleset/repository|moduleset/include">
  </xsl:template>

  <xsl:template match="moduleset/metamodule">
    <dt id="{generate-id(.)}" class="metamodule"><xsl:value-of select="@id" /></dt>
    <dd>
      <table>
        <xsl:if test="dependencies/dep">
          <tr>
            <th>Dependencies:</th>
            <td><xsl:apply-templates select="dependencies" /></td>
          </tr>
        </xsl:if>
      </table>
    </dd>
  </xsl:template>

  <xsl:template match="patch">
    <li><xsl:value-of select="@file" /></li>
  </xsl:template>

</xsl:stylesheet>
