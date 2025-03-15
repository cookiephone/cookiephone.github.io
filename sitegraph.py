import os
import json
import argparse
from bs4 import BeautifulSoup

def extract_links_from_html(file_path):
    with open(file_path, 'r', encoding='utf-8') as file:
        content = file.read()
    soup = BeautifulSoup(content, 'html.parser')
    links = set()
    for link in soup.find_all('a', href=True):
        href = link['href']
        if href.startswith('/') and not href.startswith('//'):
            links.add(href)
    return links

def generate_graph(output_directory):
    graph = {}
    for root, dirs, files in os.walk(output_directory):
        for file in files:
            if file.endswith('.html'):
                file_path = os.path.join(root, file)
                page_url = os.path.relpath(file_path, output_directory)
                page_url = '/' + page_url.replace('index.html', '').replace('\\', '/')
                links = extract_links_from_html(file_path)
                graph[page_url] = list(links)
    return graph

def save_graph_to_json(graph, output_path):
    with open(output_path, 'w', encoding='utf-8') as json_file:
        json.dump(graph, json_file, indent=4)

def compute_vizdata(graph):
    nodes = list(graph['graph'].keys())
    page_to_index = {page: index for index, page in enumerate(nodes)}
    edges = []
    for page, links in graph['graph'].items():
        node_out = page_to_index[page]
        for link in links:
            node_in = page_to_index[link]
            edges.append((node_out, node_in))
    graph['vizdata'] = {
        'nodes': nodes,
        'node_count': len(nodes),
        'edges': edges,
    }
    return graph

def main():
    parser = argparse.ArgumentParser(description="Generate a graph of internal links from HTML files.")
    parser.add_argument('site_directory', help="Path to the output directory containing HTML files")
    parser.add_argument('output_json', help="Path to save the generated graph JSON file")
    parser.add_argument('--vizdata', action='store_true', help="Compute additional data for graph visualization")
    args = parser.parse_args()
    graph = { 'graph': generate_graph(args.site_directory) }
    if args.vizdata:
        graph = compute_vizdata(graph)
    save_graph_to_json(graph, args.output_json)
    print(f'Graph saved to {args.output_json}')

if __name__ == '__main__':
    main()
