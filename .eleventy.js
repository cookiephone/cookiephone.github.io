const moment = require('moment');
 
module.exports = function (eleventyConfig) {
 
  eleventyConfig.addFilter('dateIso', date => {
    return moment(date).toISOString();
  });
 
  eleventyConfig.addFilter('dateReadableJp', date => {
    moment.locale('ja');
    return moment(date).utc().format('LL');
  });

  eleventyConfig.addShortcode('excerpt', article => extractExcerpt(article));

  eleventyConfig.addPassthroughCopy({ 'src/static' : '/'});

  return {
    dir: {
      input: "src",
      output: "gen"
    }
  }
};

function extractExcerpt(article) {
    const maxLength = 50;

    if (!article.hasOwnProperty('templateContent')) {
        console.warn('Failed to extract excerpt: Document has no property "templateContent".');
        return null;
    }
    
    let excerpt = null;
    const content = article.templateContent;
    
    const separatorsList = [
        { start: '<!-- Excerpt Start -->', end: '<!-- Excerpt End -->' },
        { start: /<p\b[^>]*>/, end: '</p>' }
    ];

    const isJapanese = (text) => /[\p{Script=Hiragana}\p{Script=Katakana}\p{Script=Han}]/u.test(text);
    
    separatorsList.some(separators => {
      let startPosition, endPosition, startMatch;

      if (typeof separators.start === 'string') {
        startPosition = content.indexOf(separators.start);
        startMatch = separators.start;
      } else {
        startMatch = content.match(separators.start);
        startPosition = startMatch ? startMatch.index : -1;
      }

      endPosition = content.indexOf(separators.end, startPosition + (startMatch ? startMatch[0].length : 0));

      if (startPosition !== -1 && endPosition !== -1) {
          rawExcerpt = content.substring(startPosition + (startMatch ? startMatch[0].length : separators.start.length), endPosition).trim();
          const ellipsis = isJapanese(rawExcerpt) ? "…" : "...";
          excerpt = rawExcerpt.length > maxLength ? rawExcerpt.substring(0, maxLength) + ellipsis : rawExcerpt;
          return true;
      }
    });
    
    return excerpt;
  }
